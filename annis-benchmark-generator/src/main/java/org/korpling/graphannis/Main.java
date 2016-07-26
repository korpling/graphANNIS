package org.korpling.graphannis;

import org.korpling.annis.benchmark.generator.QueryToJSON;

import annis.ql.parser.AnnisParserAntlr;
import annis.ql.parser.QueryData;

public class Main
{
  public static void main(String[] args)
  {
	API.Search search = new API.Search();

    AnnisParserAntlr parser = new AnnisParserAntlr();

    QueryData queryData = parser.parse(args[0], null);
    API.StringVector corpora = new API.StringVector("pcc2");
    String queryAsJSON = QueryToJSON.serializeQuery(queryData);
    
    API.StringVector results = search.find(corpora, queryAsJSON);
    long numOfResults = results.size();
    for(long i=0; i < numOfResults; i++)
    {
    	System.out.println(results.get(i).getString());
    }
    System.out.println("Count: " + search.count(corpora, QueryToJSON.serializeQuery(queryData)));

    search.close();
  }
}
