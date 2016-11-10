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

import java.util.Arrays;
import java.util.HashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import org.corpus_tools.salt.SALT_TYPE;
import org.corpus_tools.salt.common.SDocumentGraph;
import org.corpus_tools.salt.common.SDominanceRelation;
import org.corpus_tools.salt.common.SPointingRelation;
import org.corpus_tools.salt.common.SStructure;
import org.corpus_tools.salt.common.SStructuredNode;
import org.corpus_tools.salt.common.STextualDS;
import org.corpus_tools.salt.common.STextualRelation;
import org.corpus_tools.salt.common.SToken;
import org.corpus_tools.salt.core.SAnnotation;
import org.corpus_tools.salt.core.SLayer;
import org.corpus_tools.salt.core.SNode;
import org.corpus_tools.salt.core.SRelation;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * A class which helps to import salt documents into graphANNIS.
 *
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class SaltImport
{

  public static final String ANNIS_NS = "annis4_internal";

  private static final Logger log = LoggerFactory.getLogger(SaltImport.class);

  private final API.GraphUpdate updateList = new API.GraphUpdate();
  
  public API.GraphUpdate finish()
  {
    return updateList;
  }
  
  
  public SaltImport map(SDocumentGraph g)
  {
    API.GraphUpdate u = new API.GraphUpdate();

    // add all nodes and their annotations
    for (SNode n : g.getNodes())
    {
      addNode(n);
    }

    Map<SToken, Long> token2Index = addTokenInformation(g);
    
    for(SNode n : g.getNodes())
    {
      addLeftRightToken(n, g, token2Index);
    }
      
    for (SDominanceRelation rel : g.getDominanceRelations())
    {
      String sourceName = nodeName(rel.getSource());
      String targetName = nodeName(rel.getTarget());

      for (String l : getLayerNames(rel.getLayers()))
      {
        // add an edge both for the named component and for the un-named
        u.addEdge(sourceName, targetName, l, "DOMINANCE", rel.getType());
        u.addEdge(sourceName, targetName, l, "DOMINANCE", "");
      }
    }

    for (SPointingRelation rel : g.getPointingRelations())
    {
      String sourceName = nodeName(rel.getSource());
      String targetName = nodeName(rel.getTarget());

      for (String l : getLayerNames(rel.getLayers()))
      {
        // add an edge both for the named component and for the un-named
        u.addEdge(sourceName, targetName, l, "POINTING", rel.getType());
      }
    }

    return this;
  }

  private Map<SToken, Long> addTokenInformation(SDocumentGraph g)
  {
    Map<SToken, Long> token2Index = new HashMap<>();

    SToken lastToken = null;
    STextualDS lastTextDS = null;

    List<SToken> sortedToken = g.getSortedTokenByText();

    if (sortedToken != null)
    {
      long i = 0;
      for (SToken t : sortedToken)
      {
        token2Index.put(t, i++);

        String nodeName = nodeName(t);
        // each token must have it's spanned text as label
        updateList.addNodeLabel(nodeName, ANNIS_NS, "tok", g.getText(t));

        STextualDS textDS = getTextForToken(t);
        if (lastToken != null && textDS == lastTextDS)
        {
          // add an explicit ORDERING edge between the token
          updateList.addEdge(nodeName(lastToken), nodeName, ANNIS_NS, "ORDERING", "");
        }
        lastToken = t;
        lastTextDS = textDS;
      }
    }
    return token2Index;
  }

  /**
   * Add an explicit edge to the left- and right-most covered token of the node.
   *
   * @param node
   * @param graph
   * @param token2Index
   */
  private void addLeftRightToken(SNode node, SDocumentGraph graph, Map<SToken, Long> token2Index)
  {
    List<SToken> overlappedToken;
    if (node instanceof SToken)
    {
      overlappedToken = Arrays.asList((SToken) node);
    }
    else if (node instanceof SStructure)
    {
      overlappedToken = graph.getOverlappedTokens(node, SALT_TYPE.SSPANNING_RELATION,
        SALT_TYPE.SDOMINANCE_RELATION);
    }
    else
    {
      overlappedToken = graph.getOverlappedTokens(node, SALT_TYPE.SSPANNING_RELATION);

    }
    if (overlappedToken.isEmpty())
    {
      log.warn("Node {} is not connected to any token. This is invalid for graphANNIS and the node will be excluded.", node.getId());
      return;
    }

    // sort the token by left index
    List<SToken> sortedOverlappedToken = graph.getSortedTokenByText(overlappedToken);

    
    String name = nodeName(node);
    String firstOverlappedToken = nodeName(sortedOverlappedToken.get(0));
    String lastOverlappedToken = nodeName(sortedOverlappedToken.get(sortedOverlappedToken.size() - 1));

    updateList.addEdge(firstOverlappedToken, name, ANNIS_NS, "LEFT_TOKEN", "");
    updateList.addEdge(name, firstOverlappedToken, ANNIS_NS, "LEFT_TOKEN", "");
    
    updateList.addEdge(lastOverlappedToken, name, ANNIS_NS, "RIGHT_TOKEN", "");
    updateList.addEdge(name, lastOverlappedToken, ANNIS_NS, "RIGHT_TOKEN", "");

  }
  
  private static String nodeName(SNode node)
  {
    if(node != null)
    {
      return node.getPath().fragment();
    }
    else
    {
      return null;
    }
  }

  private static STextualDS getTextForToken(SToken t)
  {
    List<SRelation> out = t.getOutRelations();
    if (out != null)
    {
      for (SRelation<?, ?> rel : out)
      {
        if (rel instanceof STextualRelation)
        {
          return ((STextualRelation) rel).getTarget();
        }
      }
    }
    return null;
  }

  private static Set<String> getLayerNames(Set<SLayer> layers)
  {
    Set<String> result = new LinkedHashSet<>();

    if (layers == null || layers.isEmpty())
    {
      // add the edge to the default empty layer
      result.add("");
    }
    else
    {
      for (SLayer l : layers)
      {
        result.add(l.getName());
      }
    }

    return result;
  }

  private void addNode(SNode n)
  {
    if (n instanceof SStructuredNode)
    {
      // use the unique name
      String name = nodeName(n);
      updateList.addNode(name);
      // add all annotations
      for (SAnnotation anno : n.getAnnotations())
      {
        updateList.addNodeLabel(name, anno.getNamespace(), anno.getName(), anno.getValue_STEXT());
      }

      // add the document name if given
      String[] segments = n.getPath().segments();
      if (segments.length > 0)
      {
        updateList.addNodeLabel(name, ANNIS_NS, "document", segments[segments.length - 1]);
      }
    }
  }

}
