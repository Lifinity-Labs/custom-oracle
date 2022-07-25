import { Cluster, Connection, Keypair, PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
// @ts-ignore
import { struct, u8, u32, ns64, nu64 } from 'buffer-layout'
import { getPoolList } from "@lifinity/sdk";
import { BasicRunnable, sleep } from "../../lib/common";
import { Jupiter, RouteInfo, SplitTradeAmm, TOKEN_LIST_URL } from "@jup-ag/core";
import assert from "assert";
import fetch from 'node-fetch';

const OUTER_INTERVAL = 10_000;

const TX_DURATION = 120_000;

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

function getMarkPrice(bid: number, ask: number) {
    return (bid + ask) / 2.0;
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

export interface Token {
    chainId: number;
    address: string;
    symbol: string;
    name: string;
    decimals: number;
    logoURI: string;
    tags: string[];
}

interface Config {
    connection: string
    cluster: Cluster
    payer: number[]
    pythAccount: number[]
    programId: string
    baseTokenMint: string
    quoteTokenMint: string
    baseQuantity: number
    quoteQuantity: number
    slippage: number
    fetchInterval: number
    updateThreshold: number
    updateInterval: number
    confidence: number
    inactiveDuration: number
    minThreshold: number
    maxThreshold: number
    allowNegativeSpread: boolean
}

class Main extends BasicRunnable<Config> {

    private connection!: Connection;
    private payer!: Keypair;
    private pythAccount!: Keypair;
    private programId!: PublicKey;
    private baseTokenMint!: PublicKey;
    private quoteTokenMint!: PublicKey;
    private baseQuantity!: number;
    private quoteQuantity!: number;
    private slippage!: number;

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

    async trade(config: Config) {
        const pools = getPoolList();
        if (!pools['LFNTY-USDC']) {
            this.logger.error("cannot find LFNTY-USDC pool")
            return;
        }
    }

    async resolveTokenAccount(mint: PublicKey, tokenAccount: string | undefined): Promise<PublicKey> {
        if (tokenAccount != null) {
            return new PublicKey(tokenAccount);
        }

        const accounts = await this.connection.getTokenAccountsByOwner(this.payer.publicKey, { mint });
        assert(accounts.value.length === 1);
        return accounts.value[0].pubkey;
    }

    async getBestRoute(jupiter: Jupiter, src: PublicKey, dst: PublicKey, inputAmount: number, slippage: number): Promise<RouteInfo | undefined> {
        const route = await jupiter.computeRoutes({
            inputMint: src,
            outputMint: dst,
            inputAmount,
            slippage,
            forceFetch: true,
        });

        for (const ri of route.routesInfos) {
            if (ri.marketInfos.length === 1) {
                if (!(ri.marketInfos[0].amm instanceof SplitTradeAmm)) {
                    return ri;
                }
            }
        }
    }

    async updatePrice(price: number, status: number, now: number): Promise<void> {
        this.logger.debug(`updating: price=${price.toFixed(8)}, status=${status}, now=${now}`);

        const tx = composeUpdatePriceTransaction(this.connection, this.programId, this.pythAccount.publicKey, price, this.confidence, status);
        const { blockhash } = await this.connection.getLatestBlockhash();
        tx.recentBlockhash = blockhash;
        tx.sign(this.payer, this.pythAccount);
        
        transactionSenderAndConfirmationWaiter(this.connection, tx).then(({txid, transactionResponse}) => {
            this.logger.debug(`executed update tx: ${txid}`);
        }).catch();

        this.lastPrice = price;
        this.lastStatus = status;
        this.lastUpdate = now;
    }

    needUpdate(price: number, now: number): boolean {
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

        this.baseTokenMint = new PublicKey(config.baseTokenMint);
        this.quoteTokenMint = new PublicKey(config.quoteTokenMint);
        this.baseQuantity = config.baseQuantity;
        this.quoteQuantity = config.quoteQuantity;
        this.slippage = config.slippage;

        this.fetchInterval = config.fetchInterval;
        this.updateThreshold = config.updateThreshold;
        this.updateInterval = config.updateInterval;

        this.confidence = config.confidence;

        this.inactiveDuration = config.inactiveDuration;
        this.minThreshold = config.minThreshold;
        this.maxThreshold = config.maxThreshold;
        this.allowNegativeSpread = config.allowNegativeSpread;

        if (this.inactiveDuration && this.inactiveDuration < this.updateInterval) {
            this.logger.error("inappropriate config: 'inactiveDuration' must be longer than 'updateInterval'");
            return;
        }

        const tokens: Token[] = await (await fetch(TOKEN_LIST_URL[config.cluster])).json();

        const baseToken = tokens.find(t => t.address === config.baseTokenMint);
        const quoteToken = tokens.find(t => t.address === config.quoteTokenMint);

        if (!baseToken || !quoteToken) {
            this.logger.error("token(s) not supported by Jupiter");
            return;
        }

        this.logger.info(`starting oracle update script for ${baseToken.symbol}-${quoteToken.symbol}...`);
        this.logger.info(`params: fetchInterval=${this.fetchInterval}, updateThreshold=${this.updateThreshold}, updateInterval=${this.updateInterval}, confidence=${this.confidence}`);
        this.logger.info(`params: inactiveDuration=${this.inactiveDuration}, minThreshold=${this.minThreshold}, maxThreshold=${this.maxThreshold}, allowNegativeSpread=${this.allowNegativeSpread}`);

        while (!this._done) {
            try {
                this.connection = new Connection(config.connection);

                const jupiter = await Jupiter.load({
                    connection: this.connection,
                    cluster: config.cluster,
                    user: this.payer,
                });

                while (!this._done) {
                    const now = Date.now();

                    const routeInfo0 = await this.getBestRoute(jupiter, this.baseTokenMint, this.quoteTokenMint, this.baseQuantity, this.slippage);
                    const routeInfo1 = await this.getBestRoute(jupiter, this.quoteTokenMint, this.baseTokenMint, this.quoteQuantity, this.slippage);

                    if (routeInfo0 && routeInfo1) {
                        const bid = routeInfo0.outAmount / routeInfo0.inAmount;
                        const ask = routeInfo1.inAmount / routeInfo1.outAmount;
                        const price = getMarkPrice(bid, ask);

                        this.logger.debug(`bid ${bid.toFixed(6)}, ask ${ask.toFixed(6)}, mark ${price.toFixed(6)}`);

                        if (this.needUpdate(price, now)) {
                            const negativeSpread = bid > ask;
                            const outOfThreshold = price < this.minThreshold || this.maxThreshold < price;
                            const status = ((!this.allowNegativeSpread && negativeSpread) || outOfThreshold ? 0 : 1);
                            await this.updatePrice(price, status, now);
                        }
                    }

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
