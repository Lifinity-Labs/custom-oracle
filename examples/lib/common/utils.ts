import fs from "fs";
import toml from 'toml';

export const UserAgentString = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.67 Safari/537.36';

export async function sleep(ms: number): Promise<void> {
    return new Promise(r => setTimeout(r, ms));
}

export function readToml(path: string): any {
    try {
        const text = fs.readFileSync(path, 'utf8');
        return toml.parse(text);
    }
    catch (e) {
        return null;
    }
}
