import * as path from 'path';
import * as vscode from 'vscode';
import { execSync } from 'child_process';
import { Executable, LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node'

let client: LanguageClient;


export class ClangDocumentFormattingEditProvider implements vscode.DocumentFormattingEditProvider {
    public provideDocumentFormattingEdits(document: vscode.TextDocument, options: vscode.FormattingOptions, token: vscode.CancellationToken) {
        const fullRange = new vscode.Range(
            document.positionAt(0),
            document.positionAt(document.getText().length)
        );

        const formatted = this.invoke_clang(document.getText(), document.fileName);

        return [vscode.TextEdit.replace(fullRange, formatted)];
    }

    private invoke_clang(code: string, workspace: string): string {
        const clang_path = path.join(__dirname, '../clang-format.exe');

        try {
            return execSync(clang_path, { input: code, cwd: path.dirname(workspace) }).toString();
        } catch (e: any) {
            vscode.window.showErrorMessage(`ClangFormat failed: ${e.message || e}`);
            return code;
        }
    }

}

export function activate(ctx: vscode.ExtensionContext): void {
    let formatter = new ClangDocumentFormattingEditProvider();
    ctx.subscriptions.push(vscode.languages.registerDocumentFormattingEditProvider("cstyle", formatter))

    const command = process.env.SERVER_PATH || "c-style-lang-server";
    const run: Executable = {
        command,
        options: {
            env: {
                ...process.env,
                RUST_LOG: "debug",
            },
        },
    };
    const serverOptions: ServerOptions = {
        run,
        debug: run,
    };
    let clientOptions: LanguageClientOptions = {
        // Register the server for plain text documents
        documentSelector: [{ scheme: "file", language: "cstyle" }],
    };


    client = new LanguageClient("c-style-lang-server", "c style lang server", serverOptions, clientOptions);
    client.start();
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}