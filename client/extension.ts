import * as path from 'path';
import * as vscode from 'vscode';
import { execSync } from 'child_process';
import { Executable, LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node'

let client: LanguageClient;
let sharedTerminal: vscode.Terminal | undefined;


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

    const showRefCmd = vscode.commands.registerCommand('cstyle-lang-server.showReferences', (uri, position, locations) => {
        vscode.commands.executeCommand(
            "editor.action.showReferences",
            vscode.Uri.parse(uri),
            client.protocol2CodeConverter.asPosition(position),
            locations.map(client.protocol2CodeConverter.asLocation),
        );
    });
    ctx.subscriptions.push(showRefCmd);

    // TODO lang specific - setup a reasonable command runner
    const runMainCmd = vscode.commands.registerCommand('cstyle-lang-server.runMain', (uri, position, locations) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor found!');
            return;
        }

        if (!sharedTerminal) {
            sharedTerminal = vscode.window.createTerminal('CStyle Code Runner');
            // Dispose reference when terminal is closed
            vscode.window.onDidCloseTerminal((closedTerminal) => {
                if (closedTerminal === sharedTerminal) {
                    sharedTerminal = undefined;
                }
            });
        }

        sharedTerminal.show();

        // Show and run command
        const filePath = editor.document.fileName;
        const fileNameWithoutExt = path.basename(filePath).split('.').slice(0, -1).join('.') || path.basename(filePath);
        const dirName = path.dirname(filePath);


        // Commands to compile and run
        // Using gcc: compile to same directory
        const compileCmd = `gcc -x c "${filePath}" -o "${path.join(dirName, fileNameWithoutExt)}"`;
        const runCmd = `${path.join(dirName, fileNameWithoutExt)}.exe`;

        // Send commands to terminal
        sharedTerminal.sendText(compileCmd);
        sharedTerminal.sendText(runCmd);

    });
    ctx.subscriptions.push(runMainCmd);
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}