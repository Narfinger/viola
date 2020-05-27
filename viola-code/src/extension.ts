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


	let pausing = vscode.commands.registerCommand('viola-code.pause', () => {
		axios.post("http://localhost:8088/transport/", { t: "Pausing" });
	});
	let playing = vscode.commands.registerCommand('viola-code.play', () => {
		axios.post("http://localhost:8088/transport/", { t: "Playing" });
	});
	let repeat = vscode.commands.registerCommand('viola-code.repeat', () => {
		axios.post("http://localhost:8088/repeat/");
	});

	subscriptions.push(pausing);
	subscriptions.push(playing);
	subscriptions.push(repeat);

	// update status bar item once at start
	updateStatusBarItem();
	setInterval(updateStatusBarItem,1000 * 10);
}

function updateStatusBarItem(): void {
	myStatusBarItem.show();
	axios.all([axios.get("http://localhost:8088/transport/"), axios.get("http://localhost:8088/current_track/")]).then(axios.spread((...response: any) => {
		const transport = response[0].data;
		const track = response[1].data;
		console.log(response[1]);
		myStatusBarItem.text = transport + ": " + track.title + " by "+ track.artist;
		myStatusBarItem.show();
	})).catch(error => {
		console.log(error);
		myStatusBarItem.text = "We found some error";
	});
}