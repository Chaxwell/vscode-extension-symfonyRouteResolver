import { SidecarProcess } from './SidecarProcess';
import { Configuration } from './configuration';

/**
 * Orchestrates route indexation. Prevents concurrent runs and exposes
 * a simple `index()` method used both at startup and on PHP file saves.
 */
export class RouteIndexer {
    private indexing = false;

    constructor(
        private readonly sidecar: SidecarProcess,
        private readonly getConfig: () => Configuration
    ) {}

    /**
     * Triggers a full (re-)indexation.
     * @returns The number of routes successfully indexed.
     * @throws When the workspace is not configured or the sidecar fails.
     */
    async index(): Promise<number> {
        if (this.indexing) {
            return 0;
        }

        const config = this.getConfig();

        if (!config.workspace) {
            throw new Error('No workspace folder is open.');
        }

        this.indexing = true;
        try {
            const result = await this.sidecar.index(
                config.workspace,
                config.symfonyBinaryPath
            );
            return result.count;
        } finally {
            this.indexing = false;
        }
    }
}
