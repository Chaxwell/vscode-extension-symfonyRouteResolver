import { ChildProcess, spawn } from 'child_process';
import * as readline from 'readline';

import { ErrorResponse, IndexedResponse, SearchedResponse, SidecarResponse } from './types';

type IndexRequest = { type: 'index'; workspace_path: string; symfony_binary: string };
type SearchRequest = { type: 'search'; content: string };
type SidecarRequest = IndexRequest | SearchRequest;

type PendingEntry = {
    resolve: (r: SidecarResponse) => void;
    reject: (e: Error) => void;
};

/**
 * Manages the lifecycle of the Rust sidecar process and provides a typed
 * request/response interface over newline-delimited JSON (NDJSON).
 */
export class SidecarProcess {
    private process: ChildProcess | null = null;
    private pendingRequests = new Map<number, PendingEntry>();
    private nextId = 1;

    constructor(private readonly binaryPath: string) {}

    start(): void {
        this.process = spawn(this.binaryPath, [], {
            stdio: ['pipe', 'pipe', 'pipe'],
        });

        const rl = readline.createInterface({ input: this.process.stdout! });
        rl.on('line', (line) => this.handleLine(line));

        this.process.stderr?.on('data', (data: Buffer) => {
            console.error('[symfony-route-resolver sidecar]', data.toString());
        });

        this.process.on('error', (err) => {
            this.rejectAllPending(err);
        });

        this.process.on('exit', (code) => {
            this.rejectAllPending(
                new Error(`Sidecar process exited unexpectedly (code ${code})`)
            );
            this.process = null;
        });
    }

    async index(workspacePath: string, symfonyBinary: string): Promise<IndexedResponse> {
        const response = await this.send({ type: 'index', workspace_path: workspacePath, symfony_binary: symfonyBinary });
        if (response.type === 'error') {
            throw new Error((response as ErrorResponse).message);
        }
        return response as IndexedResponse;
    }

    async search(content: string): Promise<SearchedResponse> {
        const response = await this.send({ type: 'search', content });
        if (response.type === 'error') {
            throw new Error((response as ErrorResponse).message);
        }
        return response as SearchedResponse;
    }

    private send(request: SidecarRequest): Promise<SidecarResponse> {
        const proc = this.process;
        if (!proc?.stdin) {
            return Promise.reject(new Error('Sidecar process is not running'));
        }

        const id = this.nextId++;
        const payload = JSON.stringify({ ...request, id }) + '\n';

        return new Promise<SidecarResponse>((resolve, reject) => {
            this.pendingRequests.set(id, { resolve, reject });
            proc.stdin!.write(payload);
        });
    }

    private handleLine(raw: string): void {
        let response: SidecarResponse;
        try {
            response = JSON.parse(raw) as SidecarResponse;
        } catch {
            return;
        }

        const pending = this.pendingRequests.get(response.id);
        if (!pending) {
            return;
        }
        this.pendingRequests.delete(response.id);
        pending.resolve(response);
    }

    private rejectAllPending(err: Error): void {
        for (const [, entry] of this.pendingRequests) {
            entry.reject(err);
        }
        this.pendingRequests.clear();
    }

    dispose(): void {
        this.process?.kill();
        this.process = null;
        this.rejectAllPending(new Error('Sidecar process disposed'));
    }
}
