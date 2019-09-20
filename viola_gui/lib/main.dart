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
  final String album;
  final int length;
  final String genre;
  final int year;

  Track.fromJson(Map<String, dynamic> json)
      : title = json['title'],
        artist = json['artist'],
        album = json['album'],
        length = json['length'],
        genre = json['genre'],
        year = json['year'];
}

Future<String> loadAsset(BuildContext context) async {
  return DefaultAssetBundle.of(context).loadString('assets/tracks.json');
}

Future<List<Track>> fetchTracks(BuildContext context) async {
  var ass = await loadAsset(context);
  var l = json.decode(ass) as List;
  List<Track> tracks = l.map((i) => Track.fromJson(i)).toList();
  return tracks;
}

class MyApp extends StatelessWidget {
  Widget _buildEntry(BuildContext context, String t) {
    return Align(
        alignment: Alignment.centerLeft,
        child: Text(t,
            textAlign: TextAlign.left,
            style: Theme.of(context).textTheme.headline)
        //color: Colors.teal[100],
        );
  }

  Widget _buildGrid(BuildContext context) {
    return Column(
      children: <Widget>[
        Row(
          children: <Widget>[
            Container(child: Text("Title")),
            Container(child: Text("Artist")),
            Container(child: Text("Album")),
          ],
        ),
        FutureBuilder<List<Track>>(
            future: fetchTracks(context),
            builder: (context, snapshot) {
              if (snapshot.hasError) print(snapshot.error);
              return snapshot.hasData
                  ? Expanded(
                      child: GridView.builder(
                          itemCount: snapshot.data.length * 3,
                          gridDelegate:
                              new SliverGridDelegateWithFixedCrossAxisCount(
                                  mainAxisSpacing: 4.0,
                                  crossAxisCount: 3,
                                  childAspectRatio: 15.0 / 1.0),
                          itemBuilder: (BuildContext context, int index) {
                            int i = (index / 3).floor();
                            switch (index % 3) {
                              case 0:
                                return _buildEntry(
                                    context, snapshot.data[i].title);
                                break;
                              case 1:
                                return _buildEntry(
                                    context, snapshot.data[i].artist);
                                break;
                              case 2:
                                return _buildEntry(
                                    context, snapshot.data[i].album);
                                break;
                              case 3:
                                return _buildEntry(context, "NI");
                                break;
                              case 4:
                                return _buildEntry(
                                    context, snapshot.data[i].genre);
                                break;
                              case 5:
                                return _buildEntry(context, "NI");
                                break;
                            }
                          }))
                  : Center(child: CircularProgressIndicator());
            })
      ],
    );
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
            body: _buildGrid(context)));
  }
}
