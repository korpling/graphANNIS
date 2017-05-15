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

import static annis.service.objects.SubgraphFilter.token;
import java.util.ArrayList;
import java.util.List;
import org.corpus_tools.salt.graph.Graph;
import org.corpus_tools.salt.graph.Label;
import org.corpus_tools.salt.graph.Layer;
import org.corpus_tools.salt.graph.Node;
import org.corpus_tools.salt.graph.Relation;
import org.corpus_tools.salt.graph.impl.GraphImpl;
import org.corpus_tools.salt.graph.impl.LabelImpl;
import org.corpus_tools.salt.graph.impl.NodeImpl;
import org.corpus_tools.salt.graph.impl.RelationImpl;

/**
 * Allows to extract a Salt-Graph from a database subgraph.
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class SaltExport 
{
  
  private static List<Node> extractNodes(API.Graph orig)
  {
    ArrayList<Node> nodes = new ArrayList<>();
    
    for(long i=0; i < orig.nodes().size(); i++)
    {
      API.Node n = orig.nodes().get(i);
     
      Node newNode = new NodeImpl();
      
      for(long j=0; j < n.labels().size(); j++)
      {
        
        API.Label l = n.labels().get(j);
        Label newLabel = new LabelImpl();
        newLabel.setNamespace(l.ns().getString());
        newLabel.setName(l.name().getString());
        newLabel.setValue(l.value().getString());
        
        newNode.addLabel(newLabel);
      }
      
      nodes.add(newNode);
    }
    
    for(long i=0; i < orig.edges().size(); i++)
    {
      API.Edge e = orig.edges().get(i);
     
      Relation<Node, Node> newRel = new RelationImpl<>();

      for(long j=0; j < e.labels().size(); j++)
      {
        
        API.Label l = e.labels().get(j);
        Label newLabel = new LabelImpl();
        newLabel.setNamespace(l.ns().getString());
        newLabel.setName(l.name().getString());
        newLabel.setValue(l.value().getString());
        
        newRel.addLabel(newLabel);
      }
      
    }
    
    return nodes;
  }
  
  static Graph<Node, Relation<Node, Node>, Layer<Node,Relation<Node,Node>>> mapToBasicGraph(API.Graph orig)
  {
    Graph<Node, Relation<Node, Node>, Layer<Node,Relation<Node,Node>>>  g  = new GraphImpl<>();
    
    return g;
  }
}
