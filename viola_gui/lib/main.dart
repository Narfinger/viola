import 'package:flutter/material.dart';
import 'dart:convert';
import 'dart:async' show Future;
import 'package:flutter/services.dart' show rootBundle;

void main() {
  runApp(new MyApp());
}

class Track {
  final String title;
  final String artist;

  Track.fromJson(Map<String, dynamic> json)
      : title = json['title'],
        artist = json['artist'];
  Map<String, dynamic> toJson() => {
        'title': title,
        'artist': artist,
      };
}

Future<String> loadAsset(BuildContext context) async {
  return DefaultAssetBundle.of(context).loadString('assets/tracks.json');
}

Future<List<Track>> fetchTracks(BuildContext context) async {
  var ass = await loadAsset(context);
  List<Track> tracks = json.decode(ass).map((i) => Track.fromJson(i)).toList();
  return tracks;
}

class MyApp extends StatelessWidget {
  Widget _buildEntry(String t) {
    return Container(
      padding: const EdgeInsets.all(1),
      child: Text(t),
      color: Colors.teal[100],
    );
  }

  Widget _buildGrid(List<Track> tracks) {
    return Expanded(
        child: Column(
      children: <Widget>[
        Column(
          children: <Widget>[
            Container(child: Text("Title")),
            Container(child: Text("Artist"))
          ],
        ),
        GridView.builder(
          itemCount: tracks.length * 2,
          gridDelegate:
              new SliverGridDelegateWithFixedCrossAxisCount(crossAxisCount: 2),
          itemBuilder: (BuildContext context, int index) {
            int i = (index / 2).floor();
            switch (index % 2) {
              case 0:
                _buildEntry(tracks[i].title);
                break;
              case 1:
                _buildEntry(tracks[i].artist);
                break;
            }
          },
        )
      ],
    ));
  }

  Widget playbackcontrols = Row(children: <Widget>[
    RaisedButton(
        onPressed: () {}, child: Text('Play', style: TextStyle(fontSize: 20))),
    RaisedButton(
        onPressed: () {}, child: Text('Pause', style: TextStyle(fontSize: 20))),
    RaisedButton(
        onPressed: () {},
        child: Text('Previous', style: TextStyle(fontSize: 20))),
    RaisedButton(
        onPressed: () {}, child: Text('next', style: TextStyle(fontSize: 20))),
  ]);

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
        title: 'Viola Beta',
        home: Scaffold(
            appBar: AppBar(
              title: Text('Viola Beta'),
            ),
            body: FutureBuilder<List<Track>>(
                future: fetchTracks(context),
                builder: (context, snapshot) {
                  if (snapshot.hasError) print(snapshot.error);
                  return snapshot.hasData
                      ? this._buildGrid(snapshot.data)
                      : Center(child: CircularProgressIndicator());
                })));
  }
}
