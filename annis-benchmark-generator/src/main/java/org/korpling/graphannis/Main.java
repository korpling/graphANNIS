package org.korpling.graphannis;

import org.korpling.annis.benchmark.generator.QueryToJSON;
import org.korpling.graphannis.Annis.API;

import annis.ql.parser.AnnisParserAntlr;
import annis.ql.parser.QueryData;

public class Main
{
  public static void main(String[] args)
  {
    Annis.API api = new API();

    AnnisParserAntlr parser = new AnnisParserAntlr();

    QueryData queryData = parser.parse(args[0], null);
    System.out.println("Count: " + api.count("pcc2", QueryToJSON.serializeQuery(queryData)));

    api.close();
  }
}
