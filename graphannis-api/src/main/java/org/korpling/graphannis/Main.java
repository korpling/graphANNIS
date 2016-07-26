package org.korpling.graphannis;

import java.util.LinkedList;
import java.util.List;

import annis.ql.parser.AnnisParserAntlr;
import annis.ql.parser.QueryData;

public class Main
{
  public static void main(String[] args)
  {
    if(args.length < 2)
    {
      System.out.println("Usage: Main <corpus> [<corpus>...] <query>");
      return;
    }
    
    String databaseDir = "./data/";
    if (System.getenv("ANNIS4_TEST_DATA") != null)
    {
      databaseDir = System.getenv("ANNIS4_TEST_DATA");
    }
    API.Search search = new API.Search(databaseDir);

    AnnisParserAntlr parser = new AnnisParserAntlr();

    QueryData queryData = parser.parse(args[args.length-1], null);
    
    List<String> corpusList = new LinkedList<>();
    for(int i=0; i < args.length-1; i++)
    {
      corpusList.add(args[i]);
    }
    API.StringVector corpora = new API.StringVector(corpusList.toArray(new String[0]));
    
    
    String queryAsJSON = QueryToJSON.serializeQuery(queryData);

    API.StringVector results = search.find(corpora, queryAsJSON);
    long numOfResults = results.size();
    for (long i = 0; i < numOfResults; i++)
    {
      System.out.println(results.get(i).getString());
    }
    System.out.println("Count: " + search.count(corpora, QueryToJSON.serializeQuery(queryData)));

    search.close();
  }
}
