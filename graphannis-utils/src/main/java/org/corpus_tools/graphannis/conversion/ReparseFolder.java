/*
 * Copyright 2017 Thomas Krause.
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
package org.corpus_tools.graphannis.conversion;

import java.io.File;
import java.util.LinkedList;
import java.util.List;
import org.corpus_tools.annis.benchmark.generator.Query;
import org.corpus_tools.annis.benchmark.generator.QuerySetPersistance;
import org.corpus_tools.annis.ql.parser.AnnisParserAntlr;
import org.corpus_tools.annis.ql.parser.QueryData;
import org.corpus_tools.graphannis.QueryToJSON;

/**
 *
 * @author thomas
 */
public class ReparseFolder
{
  public static void main(String[] args)
  {
    if(args.length >= 1)
    {
      File dir = new File(args[0]);
      System.out.println("Re-parsing folder " + dir.getAbsolutePath());
      
      AnnisParserAntlr parser = new AnnisParserAntlr();
      parser.setPrecedenceBound(50);
      
      List<Query> allQueries = QuerySetPersistance.loadQuerySet(dir);
      for(Query q : allQueries)
      {
        q.setJson(null);
        QueryData queryData = parser.parse(q.getAql(), null);
        queryData.setMaxWidth(queryData.getAlternatives().get(0).size());
        String asJSON = QueryToJSON.serializeQuery(queryData.getAlternatives(), queryData.getMetaData());
        q.setJson(asJSON);
      }
      
      QuerySetPersistance.writeQuerySet(dir, allQueries);
      
    }
  }
}
