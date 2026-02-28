import * as vscode from 'vscode';

import { SidecarProcess } from './SidecarProcess';

class SymfonyRouteLink extends vscode.DocumentLink {
    constructor(
        range: vscode.Range,
        readonly filePath: string,
        readonly line: number,
        tooltip: string
    ) {
        super(range);
        this.tooltip = tooltip;
    }
}

/**
 * Provides clickable links for every Symfony route name found in the active
 * document. On click the link opens the PHP controller file at the exact
 * declaration line.
 */
export class DocumentLinkProvider
    implements vscode.DocumentLinkProvider<SymfonyRouteLink>
{
    constructor(private readonly sidecar: SidecarProcess) {}

    async provideDocumentLinks(
        document: vscode.TextDocument
    ): Promise<SymfonyRouteLink[]> {
        let response;
        try {
            response = await this.sidecar.search(document.getText());
        } catch (err) {
            // Silently swallow errors that happen before the first indexation.
            console.error('[symfony-route-resolver] search failed:', err);
            return [];
        }

        return response.matches.map((match) => {
            const start = new vscode.Position(
                match.position.start.line,
                match.position.start.col
            );
            const end = new vscode.Position(
                match.position.end.line,
                match.position.end.col
            );
            const range = new vscode.Range(start, end);
            const tooltip = `${match.route} → ${match.file.file}:${match.file.line}`;

            return new SymfonyRouteLink(range, match.file.file, match.file.line, tooltip);
        });
    }

    resolveDocumentLink(link: SymfonyRouteLink): SymfonyRouteLink {
        // Use the L{line} URI fragment so VSCode navigates directly to the
        // declaration line when the user clicks the link.
        link.target = vscode.Uri.file(link.filePath).with({
            fragment: `L${link.line}`,
        });
        return link;
    }
}
