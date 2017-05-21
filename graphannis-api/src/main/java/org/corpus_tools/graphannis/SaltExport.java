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

import com.google.common.base.Objects;
import com.google.common.collect.Multimap;
import com.google.common.collect.Range;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;
import org.apache.commons.lang3.tuple.Pair;
import org.bytedeco.javacpp.BytePointer;
import org.corpus_tools.salt.SALT_TYPE;
import org.corpus_tools.salt.SaltFactory;
import org.corpus_tools.salt.common.SDocumentGraph;
import org.corpus_tools.salt.common.SOrderRelation;
import org.corpus_tools.salt.common.SSpan;
import org.corpus_tools.salt.common.STextualDS;
import org.corpus_tools.salt.common.STextualRelation;
import org.corpus_tools.salt.common.SToken;
import org.corpus_tools.salt.core.GraphTraverseHandler;
import org.corpus_tools.salt.core.SAnnotationContainer;
import org.corpus_tools.salt.core.SFeature;
import org.corpus_tools.salt.core.SGraph;
import org.corpus_tools.salt.core.SLayer;
import org.corpus_tools.salt.core.SNode;
import org.corpus_tools.salt.core.SRelation;
import org.corpus_tools.salt.util.SaltUtil;

/**
 * Allows to extract a Salt-Graph from a database subgraph.
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class SaltExport 
{
 
  
  private static void mapLabels(SAnnotationContainer n, API.StringMap labels)
  {
    for(API.StringMap.Iterator it = labels.begin(); !it.equals(labels.end()); it = it.increment())
    {
      Pair<String, String> qname = SaltUtil.splitQName(it.first().getString());
      String value = it.second() == null ? null : it.second().getString();
      
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
  
  private static boolean hasDominanceEdge(API.Node n)
  {
    
    for(long i=0; i < n.outgoingEdges().size(); i++)
    {
      API.Edge e = n.outgoingEdges().get(i);
      
      if("DOMINANCE".equals(e.componentType().getString()))
      {
        return true;
      }
      
    }
    
    return false;
  }
  
  private static SNode mapNode(API.Node n)
  {
    SNode newNode;
    
    if(n.labels().get(new BytePointer("annis::tok")) != null)
    {
      newNode = SaltFactory.createSToken();
    }
    else if(hasDominanceEdge(n))
    {
      newNode = SaltFactory.createSStructure();
    }
    else
    {
      newNode = SaltFactory.createSSpan();
    }
    
    BytePointer nodeName = n.labels().get(new BytePointer("annis::node_name"));
    if(nodeName != null)
    {
      newNode.setId(nodeName.getString());
    }
    
    mapLabels(newNode, n.labels());
        
    return newNode;
  }
  
  private static void mapAndAddEdge(SDocumentGraph g, API.Edge origEdge, Map<Long, SNode> nodesByID)
  {
    SNode source = nodesByID.get(origEdge.sourceID());
    SNode target = nodesByID.get(origEdge.targetID());
    
    if(source != null && target != null)
    {
      SRelation<?,?> rel = null;
      switch(origEdge.componentType().getString())
      {
        case "DOMINANCE":
          rel = g.createRelation(source, target, SALT_TYPE.SDOMINANCE_RELATION, null);
          break;
        case "POINTING":
          rel = g.createRelation(source, target, SALT_TYPE.SPOINTING_RELATION, null);
          break;
        case "ORDERING":
          rel = g.createRelation(source, target, SALT_TYPE.SORDER_RELATION, null);
          break;
        case "COVERAGE":
          // only add coverage edges in salt to spans, not structures
          if(source instanceof SSpan && target instanceof SToken)
          {
            rel = g.createRelation(source, target, SALT_TYPE.SSPANNING_RELATION, null);
          }
          break;
      }
      
      if(rel != null)
      {
        if(origEdge.componentName() != null)
        {
          rel.setType(origEdge.componentName().getString());
        }
        mapLabels(rel, origEdge.labels());
        String layerName = origEdge.componentLayer() == null ? null : origEdge.componentLayer().getString();
        if(layerName != null && !layerName.isEmpty())
        {
          List<SLayer> layer = g.getLayerByName(layerName);
          if(layer == null || layer.isEmpty())
          {
            SLayer newLayer = SaltFactory.createSLayer();
            newLayer.setName(layerName);
            g.addLayer(newLayer);
            layer = Arrays.asList(newLayer);
          }
          layer.get(0).addRelation(rel);
        }
      }
    }
  }
  
  private static void recreateText(final String name, List<SNode> rootNodes, final SDocumentGraph g)
  {
    final StringBuilder text = new StringBuilder();
    final STextualDS ds = g.createTextualDS("");
    
    ds.setName(name);
    
    Map<SToken, Range<Integer>> token2Range = new HashMap<>();
    
    
    // traverse the token chain using the order relations
    g.traverse(rootNodes, SGraph.GRAPH_TRAVERSE_TYPE.TOP_DOWN_DEPTH_FIRST,
      "ORDERING_" + name,
      new GraphTraverseHandler()
    {
      @Override
      public void nodeReached(SGraph.GRAPH_TRAVERSE_TYPE traversalType,
        String traversalId, SNode currNode, SRelation<SNode, SNode> relation,
        SNode fromNode, long order)
      {
        if(fromNode != null)
        {
          text.append(" ");
        }
        
        SFeature featTok = currNode.getFeature("annis::tok");
        if(featTok != null && currNode instanceof SToken)
        {
          int idxStart = text.length();
          text.append(featTok.getValue_STEXT());
          token2Range.put((SToken) currNode, Range.closed(idxStart, text.length()));
        }
      }

      @Override
      public void nodeLeft(SGraph.GRAPH_TRAVERSE_TYPE traversalType,
        String traversalId, SNode currNode, SRelation<SNode, SNode> relation,
        SNode fromNode, long order)
      {
      }

      @Override
      public boolean checkConstraint(SGraph.GRAPH_TRAVERSE_TYPE traversalType,
        String traversalId, SRelation relation, SNode currNode,
        long order)
      {
        if(relation == null)
        {
          // TODO: check if this is ever true
          return true;
        }
        else if(relation instanceof SOrderRelation && Objects.equal(name, relation.getType()))
        {
          return true;
        }
        else
        {
          return false;
        }
      }
    });
 
    // update the actual text
    ds.setText(text.toString());
    
    // add all relations
    
    token2Range.forEach((t, r) -> 
    {
      STextualRelation rel = SaltFactory.createSTextualRelation();
      rel.setSource(t);
      rel.setTarget(ds);
      rel.setStart(r.lowerEndpoint());
      rel.setEnd(r.upperEndpoint()+1);
      g.addRelation(rel);
    });
  }
  
  
  public static SDocumentGraph map(API.NodeVector orig)
  {
    SDocumentGraph g = SaltFactory.createSDocumentGraph();
    
    // convert the vector to a map
    Map<Long, API.Node> nodesByID = new LinkedHashMap<>();
    for(long i=0; i < orig.size(); i++)
    {
      nodesByID.put(orig.get(i).id(), orig.get(i));
    }
    
    // create all new nodes
    Map<Long, SNode> newNodesByID = new LinkedHashMap<>();
    for(Map.Entry<Long, API.Node> entry : nodesByID.entrySet())
    {
      API.Node v = entry.getValue();
      SNode n = mapNode(v);
      newNodesByID.put(entry.getKey(), n);
    }
    // add them to the graph
    newNodesByID.values().stream().forEach(n -> g.addNode(n));
    
    // create and add all edges
    for(API.Node n : nodesByID.values())
    {
      for(long i=0; i < n.outgoingEdges().size(); i++)
      {
        mapAndAddEdge(g, n.outgoingEdges().get(i), newNodesByID);
      }
    }
    
    // find all chains of SOrderRelations and reconstruct the texts belonging to them
    Multimap<String, SNode> orderRoots = g.getRootsByRelationType(SALT_TYPE.SORDER_RELATION);
    for(String name : orderRoots.keySet())
    {
      ArrayList<SNode> roots = new ArrayList<>(orderRoots.get(name));
      if(SaltUtil.SALT_NULL_VALUE.equals(name))
      {
        name = null;
      }
      recreateText(name, roots , g);
    }
    
    return g;
  }
}
