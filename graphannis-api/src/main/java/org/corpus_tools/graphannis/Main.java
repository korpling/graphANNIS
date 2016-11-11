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
package org.corpus_tools.graphannis;

import annis.model.QueryNode;
import java.util.LinkedList;
import java.util.List;

/**
 *
 * @author thomas
 */
public class Main
{

  /**
   * @param args the command line arguments
   */
  public static void main(String[] args)
  {
    if(args.length > 0)
    {
      API.CorpusStorageManager search = new API.CorpusStorageManager(args[0]);
      if(args.length > 1)
      {
        List<List<QueryNode>> query = new LinkedList<>();
        List<QueryNode> alternative = new LinkedList<>();
        
        QueryNode n1 = new QueryNode(1);
        n1.setToken(true);
        
        alternative.add(n1);
        query.add(alternative);
        
        API.StringVector result = search.find(new API.StringVector(args[1]), 
          QueryToJSON.serializeQuery(query, new LinkedList<>()));
        
        for(int i=0; i < result.size(); i++)
        {
          System.out.println(result.get(i).getString());
        }
        
      }
      else
      {
        System.err.println("You need to give a corpus name as second argument.");
      }
    }
    else
    {
      System.err.println("You have to a give a database directory as argument.");
    }
  }
  
}
