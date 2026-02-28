import * as vscode from 'vscode';

export interface Configuration {
    /** Absolute path to the workspace root, or null when no folder is open. */
    workspace: string | null;
    /** Path to the `symfony` CLI binary. Defaults to `symfony` (from $PATH). */
    symfonyBinaryPath: string;
}

export function getConfiguration(): Configuration {
    const config = vscode.workspace.getConfiguration('symfony-route-resolver');

    return {
        workspace: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? null,
        symfonyBinaryPath: config.get<string>('symfonyBinaryPath') ?? 'symfony',
    };
}
