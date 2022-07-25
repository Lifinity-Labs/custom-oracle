import {readToml} from "./utils";
import {Logger} from "tslog";
import path from "path";

export interface Runnable {
    run(): void;
}

export abstract class BasicRunnable<C> implements Runnable {
    protected _done: boolean
    protected _logger: Logger
    protected _rawConfig: any

    protected get done(): boolean {
        return this._done;
    }

    protected get logger(): Logger {
        return this._logger;
    }

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
