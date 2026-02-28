import * as assert from 'assert';
import * as path from 'path';

import { SidecarProcess } from '../../SidecarProcess';

/**
 * Integration tests for SidecarProcess.
 *
 * These tests require the compiled sidecar binary to be present at
 * `sidecar/target/debug/symfony-route-resolver-sidecar`.
 * Run `cargo build` inside the `sidecar/` directory before running the tests.
 */
suite('SidecarProcess', () => {
    let sidecar: SidecarProcess;

    const binaryPath = path.resolve(
        __dirname,
        '../../../sidecar/target/debug/symfony-route-resolver-sidecar'
    );

    setup(() => {
        sidecar = new SidecarProcess(binaryPath);
        sidecar.start();
    });

    teardown(() => {
        sidecar.dispose();
    });

    test('returns an error when searching before indexing', async () => {
        try {
            await sidecar.search('some content with routes');
            assert.fail('Expected search to throw before indexing');
        } catch (err) {
            assert.ok((err as Error).message.includes('index'));
        }
    });

    test('handles malformed binary path gracefully', () => {
        const badSidecar = new SidecarProcess('/nonexistent/binary');
        assert.doesNotThrow(() => badSidecar.start());
        badSidecar.dispose();
    });
});
