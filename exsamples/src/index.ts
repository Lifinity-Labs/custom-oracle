import {BasicRunnable, sleep} from "../lib/common";

// 以下のような内容のコンフィグファイル `index.toml` を `config` ディレクトリに置いておけば、
// その内容を読み込んで表示し、数秒待機してから終了するだけのスクリプト。
// コンフィグファイルがなければ設定は読み込まずに、ただ数秒待機してから終了する。
//
// [ftx]
// apiKey = 'my api key'
// secret = 'my secret password'
//
// [solana]
// connection = 'https://ssc-dao.genesysgo.net'
// owner = [123, 45, 67, 89]
//
interface Config {
    solana: {
        connection: string
        owner: number[]
    },
    ftx: {
        apiKey: string
        secret: string
    },
}

class Main extends BasicRunnable<Config> {
    constructor() {
        super(__filename);
    }

    async main(config: Config): Promise<void> {
        if (config) {
            this.logger.info(`config:`);
            this.logger.info(`  solana: connection=${config.solana.connection}, owner=[${config.solana.owner}]`);
            this.logger.info(`  ftx: apiKey=${config.ftx.apiKey}, secret=${config.ftx.secret}`);
        }

        for (let i = 0; i < 5; ++i) {
            await sleep(1000);
            this.logger.info(`loop ${i+1}/5...`);
            if (this.done)
                break;
        }

        this.logger.info(`done - quitting`);
    }
}

new Main().run();
