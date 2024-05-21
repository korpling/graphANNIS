# Interactive command-line

The `annis` command[^rename] must be started with the data directory as argument.
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
More than one `-c` argument can be given: multiple arguments are executed in the order they where given.
E.g., `annis data -c 'set-disk-based on' -c 'import relannis/pcc2.1'` would first set the "use the disk" mode and then import the corpus with this setting.

## Commands

### `import`

The `import` command takes the directory or file to import as argument.
If this is a directory, it is assumed that the corpus is in the [relANNIS format](http://korpling.github.io/ANNIS/4.0/developer-guide/annisimportformat.html).
To import a corpus in the graphML based format, give a single file with the ending `.graphml` as argument.

You can also import a ZIP file (having the file ending `.zip`) to import multiple corpora at once.
ZIP files can contain a mixture of relANNIS and graphML files.
They also have the benefit of compression, which can be especially useful for the XML-based graphML format.

Per default, graphANNIS will keep the whole corpus in main memory for faster
query execution. You can enable the **"disk-based"** mode for a corpus by
executing the command `set-disk-based on` before the import command. This will
use much less main memory when loading a corpus, but will also cause slower
query execution. Please note that you will still need at least 4 GB of main
memory during import for larger corpora even when this option is on[^stacksize], because of
internal caching (memory usage will be less for querying the corpus).

You can also give a corpus name as an additional argument after the corpus path.
This corpus name will overwrite the automatically created name, which is e.g.
based on the information given in the imported corpus itself.



### `list`

To list the names of all imported corpora, use the `list` command.
When a corpus is loaded, it will also output the estimated memory size currently used by this corpus.

```
tiger2> list
GUM (partially loaded, 89.58 MB)
RIDGES_Herbology_Version7.0 (not loaded)
pcc2 (not loaded)
pcc2.1 (not loaded)
tiger2 (fully loaded, 1420.22 MB )
```

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

`find` also allows executing AQL queries, but instead of counting the results it will list all matching IDs.

```
GUM> find tok="Some" . pos=/N.*/
15:23:19 [ INFO] Executed query in 5 ms
GUM/GUM_interview_ants#tok_139 GUM::pos::GUM/GUM_interview_ants#tok_140
GUM/GUM_news_hackers#tok_489 GUM::pos::GUM/GUM_news_hackers#tok_490
GUM/GUM_voyage_cuba#tok_279 GUM::pos::GUM/GUM_voyage_cuba#tok_280
GUM/GUM_whow_joke#tok_657 GUM::pos::GUM/GUM_whow_joke#tok_658
GUM/GUM_whow_parachute#tok_722 GUM::pos::GUM/GUM_whow_parachute#tok_723
```

You can use the `set-limit <number>` and `set-offset <number>` commands to limit the number of matches `find` will output or to set the offset to where to output the results from.

### `frequency`

This command takes two arguments: the frequency definition and the AQL query.
The frequency definition consists of comma-separated descriptions which annotations to include in the frequency table.
Each annotation description must consist of the query node ID, followed by colon and the name of the annotation, e.g. `1:pos` to get the `pos` annotation value for the first node in the AQL query.

```
> frequency 1:pos,2:pos tok="Some" . pos=/N.*/
15:33:25 [ INFO] Executed query in 5 ms
+-------+-------+-------+
| 1#pos | 2#pos | count |
+-------+-------+-------+
| DT    | NNS   | 5     |
+-------+-------+-------+
```

### `plan`

To debug queries, you the `plan` command with the query as argument, which will output an execution plan.

```
GUM> plan tok="Some" . pos=/N.*/
15:26:20 [ INFO] Planned query in 4 ms
GUM:
+|indexjoin (parallel) (#1 . #2) [out: 92, sum: 268, instep: 268]
    #1 (tok="Some") [out: 176, sum: 0, instep: 0]
    #2 (pos=/N.*/) [out: 11461, sum: 0, instep: 0]
```

### `info`

This command will output information about the currently selected corpus, like the total main memory consumption and the memory consumption for the node annotation storage and the different edge components.
It will also output which internal implementation is used to store an edge component.

```
GUM> info
Status: "partially loaded"
Total memory: 89.58 MB
Node Annotations: 37.70 MB
------------
Component Coverage/annis/: 0 annnotations
Stats: nodes=0, avg_fan_out=0.00, max_fan_out=0, max_depth=1, tree
Implementation: AdjacencyListV1
Status: "fully loaded"
Memory: 0.00 MB
------------
Component Coverage/default_layer/: 0 annnotations
Stats: nodes=89395, avg_fan_out=7.36, max_fan_out=1867, max_depth=1
Implementation: AdjacencyListV1
Status: "fully loaded"
Memory: 14.86 MB
------------
[...]
```

A corpus might not be fully loaded into memory if not all components have been needed yet.
To load a corpus fully into main memory, use the `preload` command.

[^rename]: When downloading a binary from the release page, on MacOS you might need to rename the downloaded file from `annis.osx` to `annis`. The executable is called `annis.exe` on Windows.

[^stacksize]: For some corpora, the import process might need a lot of stack
size (a different type of main memory used by programs) and would crash during
import with an error. On Linux systems, you can run `ulimit -s unlimited` in the
shell before starting the graphANNIS CLI to allow an unlimited stack size when
the import fails otherwise.

## `delete`

Deletes the corpus with the name given as an argument.