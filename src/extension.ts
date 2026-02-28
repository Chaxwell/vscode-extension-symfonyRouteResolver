import * as fs from 'fs';
import * as path from 'path';

import * as vscode from 'vscode';

import { getConfiguration } from './configuration';
import { DocumentLinkProvider } from './DocumentLinkProvider';
import { RouteIndexer } from './RouteIndexer';
import { SidecarProcess } from './SidecarProcess';

const SIDECAR_BINARY =
    process.platform === 'win32'
        ? 'symfony-route-resolver-sidecar.exe'
        : 'symfony-route-resolver-sidecar';

/** File-glob patterns that trigger re-indexation on save. */
const PHP_GLOB = '**/*.php';

export function activate(context: vscode.ExtensionContext): void {
    console.log('coucou')
    const binaryPath = path.join(context.extensionPath, 'bin', SIDECAR_BINARY);

    if (!fs.existsSync(binaryPath)) {
        vscode.window.showErrorMessage(
            `Symfony Route Resolver: sidecar binary not found at "${binaryPath}". ` +
                'Run `npm run compile-rust` to build it.'
        );
        return;
    }

    const sidecar = new SidecarProcess(binaryPath);
    sidecar.start();

    const indexer = new RouteIndexer(sidecar, getConfiguration);

    // -----------------------------------------------------------------------
    // Document link providers
    // -----------------------------------------------------------------------

    const linkProvider = new DocumentLinkProvider(sidecar);
    const supportedLanguages = ['php', 'twig', 'yaml', 'javascript', 'javascriptreact', 'typescript', 'typescriptreact'];

    for (const language of supportedLanguages) {
        context.subscriptions.push(
            vscode.languages.registerDocumentLinkProvider({ language }, linkProvider)
        );
    }

    // -----------------------------------------------------------------------
    // Manual re-index command
    // -----------------------------------------------------------------------

    context.subscriptions.push(
        vscode.commands.registerCommand(
            'symfony-route-resolver.reindex',
            () => {
                vscode.window.withProgress(
                    {
                        location: vscode.ProgressLocation.Notification,
                        title: 'Symfony Route Resolver: indexing routes…',
                        cancellable: false,
                    },
                    async () => {
                        try {
                            const count = await indexer.index();
                            vscode.window.setStatusBarMessage(
                                `Symfony Routes: ${count} routes indexed`,
                                10000
                            );
                        } catch (err) {
                            vscode.window.showErrorMessage(
                                `Symfony Route Resolver: indexation failed — ${(err as Error).message}`
                            );
                        }
                    }
                );
            }
        )
    );

    // -----------------------------------------------------------------------
    // Re-index on PHP file save
    // -----------------------------------------------------------------------

    const phpWatcher = vscode.workspace.createFileSystemWatcher(PHP_GLOB);
    const scheduleReindex = debounce(() => {
        indexer.index().catch((err) =>
            console.error('[symfony-route-resolver] background re-index failed:', err)
        );
    }, 1500);

    phpWatcher.onDidChange(scheduleReindex, null, context.subscriptions);
    phpWatcher.onDidCreate(scheduleReindex, null, context.subscriptions);
    phpWatcher.onDidDelete(scheduleReindex, null, context.subscriptions);
    context.subscriptions.push(phpWatcher);

    // -----------------------------------------------------------------------
    // Initial indexation at startup
    // -----------------------------------------------------------------------

    indexer
        .index()
        .then((count) => {
            vscode.window.setStatusBarMessage(
                `Symfony Routes: ${count} routes indexed`,
                5000
            );
        })
        .catch((err) => {
            // Not critical — show a warning but do not block activation.
            vscode.window.showWarningMessage(
                `Symfony Route Resolver: initial indexation failed — ${(err as Error).message}`
            );
        });

    // -----------------------------------------------------------------------
    // Cleanup on deactivation
    // -----------------------------------------------------------------------

    context.subscriptions.push({ dispose: () => sidecar.dispose() });
}

export function deactivate(): void {}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function debounce<T extends unknown[]>(
    fn: (...args: T) => void,
    delayMs: number
): (...args: T) => void {
    let timer: ReturnType<typeof setTimeout> | undefined;
    return (...args: T) => {
        clearTimeout(timer);
        timer = setTimeout(() => fn(...args), delayMs);
    };
}
