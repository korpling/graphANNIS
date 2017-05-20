/*
 * Copyright 2016 Thomas Krause.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package org.corpus_tools.graphannis.console;

import com.google.common.base.Splitter;
import java.io.File;
import java.io.IOException;
import java.util.List;
import java.util.logging.Level;
import java.util.logging.Logger;
import jline.console.ConsoleReader;
import jline.console.completer.StringsCompleter;
import jline.console.history.FileHistory;
import org.bytedeco.javacpp.BytePointer;
import org.corpus_tools.graphannis.API;

import static org.corpus_tools.graphannis.QueryToJSON.aqlToJSON;
import org.corpus_tools.graphannis.SaltExport;
import org.corpus_tools.salt.common.SDocumentGraph;
import org.corpus_tools.salt.util.SaltUtil;
import org.eclipse.emf.common.util.URI;

/**
 * An interactive console for testing the graphANNIS API.
 *
 * @author Thomas Krause <thomaskrausse@posteo.de>
 */
public class Console
{

  private final API.CorpusStorageManager mgr;

  public Console(String databaseDir)
  {
    mgr = new API.CorpusStorageManager(databaseDir);
  }

  private void list()
  {
    API.StringVector result = mgr.list();
    for (long i = 0; i < result.size(); i++)
    {
      String corpus = result.get(i).getString();

      API.CorpusStorageManager.CorpusInfo info = mgr.info(corpus);
      double memoryBytes = info.memoryUsageInBytes();
      String memoryInMB = String.format("%.2f", memoryBytes / (1024.0 * 1024.0));
      System.out.println(result.get(i).getString()
        + ": " + info.loadStatus().getString() + " (" + memoryInMB + " MB)");
    }
  }

  private void count(String argsRaw)
  {
    List<String> args = Splitter.on(" ").limit(2).omitEmptyStrings().trimResults().splitToList(argsRaw);

    if (args.size() != 2)
    {
      System.out.println("You must give the corpus name and the query as argument");
      return;
    }

    long result = mgr.count(new API.StringVector(args.get(0)), aqlToJSON(args.get(1)));

    System.out.println("" + result + " results.");
  }
  
  private void find(String argsRaw)
  {
    List<String> args = Splitter.on(" ").limit(2).omitEmptyStrings().trimResults().splitToList(argsRaw);

    if (args.size() != 2)
    {
      System.out.println("You must give the corpus name and the query as argument");
      return;
    }

    API.StringVector result = mgr.find(new API.StringVector(args.get(0)), aqlToJSON(args.get(1)));
    
    for(long i=0; i < result.size(); i++)
    {
      System.out.println(result.get(i).getString());
    }

    System.out.println("" + result.size() + " results.");
  }
  
  private void subgraph(String argsRaw)
  {
    List<String> args = Splitter.on(" ").omitEmptyStrings().trimResults().splitToList(argsRaw);

    if (args.size() < 2)
    {
      System.out.println("You must give the corpus name and the node IDs as argument");
      return;
    }
    
    API.StringVector nodeIDs = new API.StringVector();
    for(int i=1; i < args.size(); i++)
    {
      nodeIDs.put(new BytePointer(args.get(i)));
    }
    
    API.NodeVector result = mgr.subgraph(new BytePointer(args.get(0)), nodeIDs, 5, 5);
    
    SDocumentGraph docGraph = SaltExport.map(result);
    SaltUtil.saveDocumentGraph(docGraph, URI.createFileURI("/tmp/graphannis.salt"));
    System.out.println("Result saved to /tmp/graphannis.salt");
  }

  private void relannis(String argsRaw)
  {
    List<String> args = Splitter.on(" ").limit(2).omitEmptyStrings().trimResults().splitToList(argsRaw);
    if (args.size() != 2)
    {
      System.out.println("You must give path to the relANNIS files  and the corpus name as argument");
      return;
    }
    
    mgr.importRelANNIS(args.get(0), args.get(1));

    System.out.println("Imported.");
  }

  public static void main(String[] args)
  {

    if (args.length < 1)
    {
      System.err.println("Must give the database directory as argument.");
      System.exit(-1);
    }

    Console c = new Console(args[0]);

    try
    {
      Splitter cmdArgSplitter = Splitter.on(" ").omitEmptyStrings().trimResults().limit(2);

      FileHistory history = new FileHistory(new File(".graphannis_history.txt"));
      ConsoleReader reader = new ConsoleReader();

      reader.setHistory(history);
      reader.setHistoryEnabled(true);
      reader.setPrompt("graphannis> ");
      reader.addCompleter(new StringsCompleter("quit", "exit", "count", "find", "subgraph", "list", "relannis"));

      boolean exit = false;

      String line;
      while (!exit && (line = reader.readLine()) != null)
      {
        List<String> parsed = cmdArgSplitter.splitToList(line);

        String cmd = parsed.get(0);
        String arguments = "";
        if (parsed.size() > 1)
        {
          arguments = parsed.get(1);
        }
        switch (cmd)
        {
          case "list":
            c.list();
            break;
          case "count":
            c.count(arguments);
            break;
          case "find":
            c.find(arguments);
            break;
          case "subgraph":
            c.subgraph(arguments);
            break;
          case "relannis":
            c.relannis(arguments);
            break;
          case "exit":
          case "quit":
            System.out.println("Good bye!");
            exit = true;
            break;
        }
      }
    }
    catch (IOException ex)
    {
      Logger.getLogger(Console.class.getName()).log(Level.SEVERE, null, ex);
    }

  }
}
