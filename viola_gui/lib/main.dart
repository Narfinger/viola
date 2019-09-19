import 'package:flutter/material.dart';

void main() {
  runApp(new MyApp());
}

class MyApp extends StatelessWidget {
  Widget _buildEntry(String t) {
    return Container(
      padding: const EdgeInsets.all(1),
      child: Text(t),
      color: Colors.teal[100],
    );
  }

  Widget _buildGrid() {
    return Expanded(
        child: GridView.count(
            scrollDirection: Axis.vertical,
            crossAxisCount: 2,
            children: <Widget>[
          this._buildEntry("t1"),
          this._buildEntry("t2"),
          this._buildEntry("t3"),
          this._buildEntry("t4"),
          this._buildEntry("t5"),
          this._buildEntry("t6"),
          this._buildEntry("t7"),
          this._buildEntry("t8"),
          this._buildEntry("t9"),
          this._buildEntry("t10"),
          this._buildEntry("t11"),
          this._buildEntry("t12"),
        ]));
  }

  Widget playbackcontrols = Row(children: <Widget>[
    const RaisedButton(
        onPressed: null,
        child: Text('Disabled Button', style: TextStyle(fontSize: 20))),
    const RaisedButton(
        onPressed: null,
        child: Text('Disabled Button2', style: TextStyle(fontSize: 20))),
    const RaisedButton(
        onPressed: null,
        child: Text('Disabled Button3', style: TextStyle(fontSize: 20))),
  ]);

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
        title: 'Viola Beta',
        home: Scaffold(
            appBar: AppBar(
              title: Text('Viola Beta'),
            ),
            body: Center(
                child: Column(children: <Widget>[
              this.playbackcontrols,
              this._buildGrid(),
            ]))));
  }
}
