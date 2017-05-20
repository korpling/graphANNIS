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
package org.corpus_tools.graphannis;

import com.google.common.collect.ImmutableMultimap;
import com.google.common.collect.Multimaps;
import java.util.LinkedList;
import java.util.List;
import org.apache.commons.lang3.tuple.Pair;
import org.bytedeco.javacpp.BytePointer;
import org.corpus_tools.salt.SaltFactory;
import org.corpus_tools.salt.common.SDocumentGraph;
import org.corpus_tools.salt.common.SToken;
import org.corpus_tools.salt.core.SNode;
import org.corpus_tools.salt.util.SaltUtil;

/**
 * Allows to extract a Salt-Graph from a database subgraph.
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class SaltExport 
{
 
  
  private static void mapLabels(SNode n, API.StringMap labels)
  {
    for(API.StringMap.Iterator it = labels.begin(); it != labels.end(); it = it.increment())
    {
      Pair<String, String> qname = SaltUtil.splitQName(it.first().getString());
      String value = it.second().getString();
      
      if("annis".equals(qname.getKey()))
      {
        n.createFeature(qname.getKey(), qname.getValue(), value);
      }
      else
      {
        n.createAnnotation(qname.getKey(), qname.getValue(), value);
      }
    }
  }
  
  private static void mapToken(SDocumentGraph g, API.Node tokenNode)
  {
    SToken t = SaltFactory.createSToken();
    
    BytePointer nodeName = tokenNode.labels().get(new BytePointer("annis::node_name"));
    if(nodeName != null)
    {
      t.setId(nodeName.getString());
    }
    
    mapLabels(t, tokenNode.labels());
    
    g.addNode(t);
  }
  
  
  public static SDocumentGraph map(API.NodeVector orig)
  {
    SDocumentGraph g = SaltFactory.createSDocumentGraph();
    
    // convert the vector to a list
    List<API.Node> nodeList = new LinkedList<>();
    for(long i=0; i < orig.size(); i++)
    {
      nodeList.add(orig.get(i));
    }
    
    ImmutableMultimap<String, API.Node> nodesByType = Multimaps.index(nodeList, (API.Node input) -> {
      BytePointer val = input.labels().get(new BytePointer("annis::node_type"));
      return val == null ? "" : val.getString();
    });
    
    final BytePointer tokKey = new BytePointer("annis::tok");
    
    // create all token
    nodeList.stream().filter(n -> n.labels().get(tokKey) != null)
            .forEach(n -> mapToken(g, n));
    
    // TODO: connect the token with ordering relations
    // TODO: create STextualDS
    // TODO: add other nodes
    // TODO: add other edges
    return g;
  }
}
