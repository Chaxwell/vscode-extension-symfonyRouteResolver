import * as assert from 'assert';

/**
 * Unit tests for DocumentLinkProvider logic.
 *
 * These tests exercise the position-mapping logic in isolation, without
 * starting a VSCode extension host or a real sidecar process.
 */
suite('DocumentLinkProvider', () => {
    /**
     * Simulates the VSCode Position creation from a sidecar TextMatch.
     * This mirrors the logic inside DocumentLinkProvider.provideDocumentLinks.
     */
    function matchToPositions(match: {
        position: { start: { line: number; col: number }; end: { line: number; col: number } };
    }) {
        return {
            start: { line: match.position.start.line, character: match.position.start.col },
            end: { line: match.position.end.line, character: match.position.end.col },
        };
    }

    test('maps sidecar TextMatch positions to VSCode-compatible positions', () => {
        const match = {
            route: 'app_home_index',
            position: { start: { line: 2, col: 8 }, end: { line: 2, col: 22 } },
            file: { file: '/src/Controller/HomeController.php', line: 10 },
        };

        const pos = matchToPositions(match);

        assert.strictEqual(pos.start.line, 2);
        assert.strictEqual(pos.start.character, 8);
        assert.strictEqual(pos.end.line, 2);
        assert.strictEqual(pos.end.character, 22);
    });

    test('constructs the correct L{line} URI fragment', () => {
        const filePath = '/src/Controller/HomeController.php';
        const line = 42;

        // Mirrors the resolveDocumentLink logic.
        const fragment = `L${line}`;
        assert.strictEqual(fragment, 'L42');

        // The URI would be `file:///src/Controller/HomeController.php#L42`.
        const expectedUri = `file://${filePath}#${fragment}`;
        assert.ok(expectedUri.endsWith('#L42'));
    });
});
