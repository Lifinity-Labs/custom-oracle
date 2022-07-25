import { Cluster, Connection, Keypair, PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
// @ts-ignore
import { struct, u8, u32, ns64, nu64 } from 'buffer-layout'
import { Marinade, MarinadeConfig } from "@marinade.finance/marinade-ts-sdk";
import { BasicRunnable, sleep } from "../../lib/common";
import assert from "assert";

const OUTER_INTERVAL = 10_000;

const TX_DURATION = 120_000; // 2 minutes

const SEND_OPTIONS = {
    skipPreflight: true,
    maxRetries: 2,
};

function getUnixTs() {
    return new Date().getTime();
}

async function transactionSenderAndConfirmationWaiter(
    connection: Connection,
    signedTransaction: Transaction,
    timeout = TX_DURATION,
    pollInterval = 1000,
    sendInterval = 5000,
    sendRetries = 40
) {
    const rawTransaction = signedTransaction.serialize();
    const txid = await connection.sendRawTransaction(
        rawTransaction,
        SEND_OPTIONS
    );
    const start = getUnixTs();
    let lastSendTimestamp = getUnixTs();
    let retries = 0;

    while (getUnixTs() - start < timeout) {
        const timestamp = getUnixTs();

        if (retries < sendRetries && timestamp - lastSendTimestamp > sendInterval) {
            lastSendTimestamp = timestamp;
            retries += 1;
            await connection.sendRawTransaction(rawTransaction, SEND_OPTIONS);
        }

        const response = await Promise.any([
            connection.getTransaction(txid, {
                commitment: "confirmed",
            }),
            sleep(1500),
        ]);
        if (response)
            return {
                txid,
                transactionResponse: response,
            };
        await sleep(pollInterval);
    }

    return {
        txid,
        transactionResponse: null,
    };
}

function composeUpdatePriceTransaction(connection: Connection, programId: PublicKey, pythAccount: PublicKey, price: number, confidence: number, status: number) {
    const transaction = new Transaction();
    const dataLayout = struct([u8('instruction'), ns64('price'), nu64('confidence'), u32('status')]);
    const keys = [
        { pubkey: pythAccount, isSigner: true, isWritable: true }
    ];

    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
        {
            instruction: 0,
            price: price * 100_000_000,
            confidence: confidence * 10_000,
            status: status,
        },
        data
    );

    transaction.add(
        new TransactionInstruction({
            keys,
            programId,
            data
        })
    );

    return transaction;
}

interface Config {
    connection: string
    cluster: Cluster
    payer: number[]
    pythAccount: number[]
    programId: string
    fetchInterval: number           // Jupiter データを取得する時間間隔
    updateThreshold: number         // オンチェーンデータを更新する価格変動 (%) の閾値
    updateInterval: number          // オンチェーンデータを強制的に更新する時間間隔
    confidence: number              // 信頼区間
    inactiveDuration: number        // これより長い時間更新されなかったら強制的にステータスを無効にする (設定値が 0 なら無視)
}

class Main extends BasicRunnable<Config> {

    private connection!: Connection;
    private payer!: Keypair;
    private pythAccount!: Keypair;
    private programId!: PublicKey;

    private fetchInterval!: number;
    private updateThreshold!: number;
    private updateInterval!: number;

    private confidence!: number;

    private inactiveDuration!: number;
    private minThreshold!: number;
    private maxThreshold!: number;
    private allowNegativeSpread!: boolean;

    private lastUpdate = 0;
    private lastPrice = 0;
    private lastStatus = 0;

    constructor() {
        super(__filename, false);
    }

    async updatePrice(price: number, status: number, now: number): Promise<void> {
        this.logger.debug(`updating: price=${price.toFixed(8)}, status=${status}, now=${now}`);

        const oraclePrice = price ? 1.0 / price : 0;

        // トランザクションの準備
        const tx = composeUpdatePriceTransaction(this.connection, this.programId, this.pythAccount.publicKey, oraclePrice, this.confidence, status);
        const { blockhash } = await this.connection.getLatestBlockhash();
        tx.recentBlockhash = blockhash;
        tx.sign(this.payer, this.pythAccount);
        
        // トランザクションの実行（2 分ほどかけて TX チェックと再送を行い、それでもダメならタイムアウト）
        // ただし、メインループの処理を実行するために、新たな Promise を起動し、本関数からは TX 完了を待たずに抜ける
        transactionSenderAndConfirmationWaiter(this.connection, tx).then(({txid, transactionResponse}) => {
            this.logger.debug(`executed update tx: ${txid}`);
        }).catch();

        this.lastPrice = price;
        this.lastStatus = status;
        this.lastUpdate = now;
    }

    needUpdate(price: number, now: number): boolean {
        // N% の変動または一定時間経過で更新
        if (now > this.lastUpdate + this.updateInterval)
            return true;

        assert(this.lastPrice);
        const deviation = Math.abs((price - this.lastPrice) / this.lastPrice * 100);
        return deviation > this.updateThreshold;
    }

    async main(config: Config): Promise<void> {
        if (!config) {
            this.logger.error("no configuration file specified");
            return;
        }

        this.payer = Keypair.fromSecretKey(Uint8Array.from(config.payer));
        this.pythAccount = Keypair.fromSecretKey(Uint8Array.from(config.pythAccount));
        this.programId = new PublicKey(config.programId);

        this.fetchInterval = config.fetchInterval;
        this.updateThreshold = config.updateThreshold;
        this.updateInterval = config.updateInterval;

        this.confidence = config.confidence;

        this.inactiveDuration = config.inactiveDuration;

        // DEBUG ログを非表示にするには、以下の行をアンコメント。
        // this.logger.setSettings({'minLevel': 'info'});

        if (this.inactiveDuration && this.inactiveDuration < this.updateInterval) {
            this.logger.error("inappropriate config: 'inactiveDuration' must be longer than 'updateInterval'");
            return;
        }

        this.logger.info(`starting mSOL oracle update script...`);
        this.logger.info(`params: fetchInterval=${this.fetchInterval}, updateThreshold=${this.updateThreshold}, updateInterval=${this.updateInterval}, confidence=${this.confidence}`);
        this.logger.info(`params: inactiveDuration=${this.inactiveDuration}, minThreshold=${this.minThreshold}, maxThreshold=${this.maxThreshold}, allowNegativeSpread=${this.allowNegativeSpread}`);

        while (!this._done) {
            try {
                // コネクションなど長時間稼働中に作り直しが必要かもしれないオブジェクトはループ内で構築。
                this.connection = new Connection(config.connection);
                const marinadeConfig = new MarinadeConfig({connection: this.connection});
                const marinade = new Marinade(marinadeConfig);

                while (!this._done) {
                    const now = Date.now();

                    const marinadeState = await marinade.getMarinadeState();
                    const { mSolPrice } = marinadeState;

                    if (mSolPrice && this.needUpdate(mSolPrice, now)) {
                        await this.updatePrice(mSolPrice, 1, now);
                    }

                    // 最終更新から時間が経っていたらデータが取得できない状態がしばらく続いている可能性があるのでステータスを無効にする
                    if (this.inactiveDuration && now > this.lastUpdate + this.inactiveDuration && this.lastStatus) {
                        this.logger.warn(`disabling oracle: no update for ${this.inactiveDuration} ms`);
                        await this.updatePrice(this.lastPrice, 0, now);
                    }

                    if (!this._done) {
                        await sleep(this.fetchInterval);
                    }
                }
            } catch (e) {
                this.logger.error(`error in main persistent loop: ${e}`);
            } finally {
                // some cleanup (if required)
            }

            if (!this._done) {
                await sleep(OUTER_INTERVAL);
            }
        }
    }
}

new Main().run();
