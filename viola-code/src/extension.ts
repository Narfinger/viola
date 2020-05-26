import * as vscode from 'vscode';
import axios from 'axios';

let myStatusBarItem: vscode.StatusBarItem;


// this method is called when your extension is activated
// your extension is activated the very first time the command is executed
export function activate({ subscriptions }: vscode.ExtensionContext) {
	const myCommandId = 'sample.showSelectionCount';

	// create a new status bar item that we can now manage
	myStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
	subscriptions.push(myStatusBarItem);

	// update status bar item once at start
	updateStatusBarItem();
	setTimeout(updateStatusBarItem,500);
}

function updateStatusBarItem(): void {
	myStatusBarItem.show();
	myStatusBarItem.text = "testsss";
	axios.get("http://localhost:8088/transport/").then((response: any) => {
		myStatusBarItem.text = response.data;
		myStatusBarItem.show();
	});
}