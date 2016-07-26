package org.korpling.graphannis;

import org.korpling.annis.benchmark.generator.QueryToJSON;
import org.korpling.graphannis.Annis.API;
import org.korpling.graphannis.Annis.StringVector;

import annis.ql.parser.AnnisParserAntlr;
import annis.ql.parser.QueryData;

public class Main
{
  public static void main(String[] args)
  {
    Annis.API api = new API();

    AnnisParserAntlr parser = new AnnisParserAntlr();

    QueryData queryData = parser.parse(args[0], null);
    Annis.StringVector corpora = new Annis.StringVector("pcc2");
    String queryAsJSON = QueryToJSON.serializeQuery(queryData);
    
    StringVector results = api.find(corpora, queryAsJSON);
    long numOfResults = results.size();
    for(long i=0; i < numOfResults; i++)
    {
    	System.out.println(results.get(i).getString());
    }
    System.out.println("Count: " + api.count(corpora, QueryToJSON.serializeQuery(queryData)));

    api.close();
  }
}
