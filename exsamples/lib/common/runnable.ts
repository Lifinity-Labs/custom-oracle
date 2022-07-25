import {readToml} from "./utils";
import {Logger} from "tslog";
import path from "path";

/**
 * 起動したら一度だけ処理を実行して終了するスクリプトのような処理を抽象化したインターフェース。
 */
export interface Runnable {
    run(): void;
}

/**
 * 自動的にスクリプト名と同名の TOML ファイルを `config` ディレクトリ以下から特定の型で読み込み、
 * 処理が完了するか Ctrl-C で停止する基本的なスクリプトの骨組みを与える抽象クラス。
 *
 * スクリプトは ts-node で実行される前提で、スクリプトファイルの拡張子は `.ts` を想定している。
 *
 * 起動時のコマンドライン引数によって TOML ファイル名のカスタマイズは可能。
 */
export abstract class BasicRunnable<C> implements Runnable {
    protected _done: boolean
    protected _logger: Logger
    protected _rawConfig: any

    /**
     * 処理の終了フラグ。派生クラスはこのフラグを定期的にチェックして、true になったら処理を中断／終了しなければならない。
     * @protected
     */
    protected get done(): boolean {
        return this._done;
    }

    /**
     * 標準ロガーオブジェクト。
     * @protected
     */
    protected get logger(): Logger {
        return this._logger;
    }

    /**
     * オリジナルのコンフィグ情報を any 型で取得するためのアクセッサ。
     * @protected
     */
    protected get rawConfig(): any {
        return this._rawConfig;
    }

    protected constructor(protected readonly _filePath: string, colorizeLogs = true) {
        const filename = path.basename(this._filePath, '.ts');
        this._logger = new Logger({
            name: filename,
            displayLoggerName: false,
            displayFunctionName: false,
            displayFilePath: 'hidden',
            dateTimeTimezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
            colorizePrettyLogs: colorizeLogs,
        });
        this._done = false;
    }

    private onInterrupt() {
        this._done = true;
    }

    run(): void {
        process.on('SIGINT', () => this.onInterrupt());

        try {
            // src/script.ts なら config/script.toml を、src/dir/script.ts なら config/dir/script.toml を読み込む。
            // TOML ファイル名をカスタマイズするには ts-node script.ts config.toml のように引数を与える。
            // npx 経由でもそうでなくても argv[2] が TOML ファイル名になる。
            let tomlFileName = this._filePath.replace(/\/src\/(.*?)\.ts/, '/config/$1.toml');
            if (process.argv.length > 2) {
                tomlFileName = path.resolve(tomlFileName, `../${process.argv[2]}`);
            }

            this._rawConfig = readToml(tomlFileName);
            const c: C = this._rawConfig;

            this.main(c).catch(e => {
                this.logger.error("uncaught error from main", e);
            });
        } catch (e) {
            this.logger.error("unhandled error in run", e);
        }
    }

    abstract main(config: C): Promise<void>;
}
