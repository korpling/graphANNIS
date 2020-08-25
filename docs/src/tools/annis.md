# Interactive CLI

The `annis` command must be started with the data directory as argument.
When running, it will provide you with an interactive prompt where you can execute commands using the generic `command-name arguments` syntax.
E.g, to list all corpora in the data directory just enter the `list` command without arguments.

```
>> list
GUM (not loaded)
RIDGES_Herbology_Version7.0 (not loaded)
pcc2 (not loaded)
pcc2.1 (not loaded)
tiger2 (not loaded)
```

The interactive CLI supports auto-completion by pressing the <kbd>Tab</kbd> key and you can navigate to old executed commands using the arrow up and down keys.

You can also use the `-c` argument at startup to execute a single command instead of starting the interactive command line.
This is useful for e.g. importing or exporting corpora from a script.

## Commands

### `import`

The `import` command takes the directory of file to import as argument.
If this is a directory, it is assumed that the corpus is in the [relANNIS format](http://korpling.github.io/ANNIS/4.0/developer-guide/annisimportformat.html).
To import a corpus in the graphML based format, give a single file with the ending `.graphml` as argument.

You can also import a ZIP file (having the file ending `.zip`) to import multiple corpora at once.
ZIP files can contain a mixture of relANNIS and graphML files.
They also have the benefit of compression, which can be especially useful for the XML-based graphML format.

### `list`

To list the names of all imported corpora, use the `list` command.

### `corpus`

Initially, the CLI will start with an empty corpus selection.
To select one or more corpora, call `corpus` with the corpus names separated by space as argument.

```
>> corpus pcc2 GUM
pcc2,GUM>
```

The prompt will change from `>>` to the list of corpus names and `>`.

### `export`

This command allows to export the currently selected corpus into a graphML file, which is given as argument.
When using the file ending `.zip` instead of `.graphml`, the graphML output will be packaged into a compressed ZIP-file.
You can also use a directory as argument, in this case all selected corpora will be exported into separate graphML files in this directory and with the corpus name as part of the file name.

### `count`

When one or more corpus is selected, you can use `count <query>` to get the number of matches for an AQL query.

```
GUM> count tok
15:18:52 [ INFO] Executed query in 13 ms
result: 44079 matches
```

### `find`

`find` also allows to execute AQL queries, but instead of counting the results it will list all matching IDs.

```
GUM> find tok="Some" . pos=/N.*/
15:23:19 [ INFO] Executed query in 5 ms
GUM/GUM_interview_ants#tok_139 GUM::pos::GUM/GUM_interview_ants#tok_140
GUM/GUM_news_hackers#tok_489 GUM::pos::GUM/GUM_news_hackers#tok_490
GUM/GUM_voyage_cuba#tok_279 GUM::pos::GUM/GUM_voyage_cuba#tok_280
GUM/GUM_whow_joke#tok_657 GUM::pos::GUM/GUM_whow_joke#tok_658
GUM/GUM_whow_parachute#tok_722 GUM::pos::GUM/GUM_whow_parachute#tok_723
```
