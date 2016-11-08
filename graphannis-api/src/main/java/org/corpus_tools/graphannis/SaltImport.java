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

import java.util.LinkedHashSet;
import java.util.Set;
import org.corpus_tools.salt.common.SDocumentGraph;
import org.corpus_tools.salt.common.SDominanceRelation;
import org.corpus_tools.salt.common.SPointingRelation;
import org.corpus_tools.salt.common.SToken;
import org.corpus_tools.salt.core.SAnnotation;
import org.corpus_tools.salt.core.SLayer;
import org.corpus_tools.salt.core.SNode;

/**
 * A class which helps to import salt documents into graphANNIS.
 * 
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class SaltImport
{
  public static API.GraphUpdate map(SDocumentGraph g)
  {
    API.GraphUpdate u = new API.GraphUpdate();
    
    for(SToken t : g.getTokens())
    {
      // use the unique name
      String name = t.getPath().fragment();
      u.addNode(name);
      // add all annotations
      for(SAnnotation anno : t.getAnnotations())
      {
        u.addNodeLabel(name, anno.getNamespace(), anno.getName(), anno.getValue_STEXT());
      }
    }
    
    for(SDominanceRelation rel : g.getDominanceRelations())
    {
      String sourceName = rel.getSource().getPath().fragment();
      String targetName = rel.getTarget().getPath().fragment();
     
      for(String l : getLayerNames(rel.getLayers()))
      {
        // add an edge both for the named component and for the un-named
        u.addEdge(sourceName, targetName, l, "DOMINANCE", rel.getType());
        u.addEdge(sourceName, targetName, l, "DOMINANCE", "");
      }
    }
    
    for(SPointingRelation rel : g.getPointingRelations())
    {
      String sourceName = rel.getSource().getPath().fragment();
      String targetName = rel.getTarget().getPath().fragment();
      
      for(String l : getLayerNames(rel.getLayers()))
      {
        // add an edge both for the named component and for the un-named
        u.addEdge(sourceName, targetName, l, "POINTING", rel.getType());
      }
    }
    
    return u;
  }
  
  private static Set<String> getLayerNames(Set<SLayer> layers)
  {
    Set<String> result = new LinkedHashSet<>();
    
    if(layers == null || layers.isEmpty())
    {
      // add the edge to the default empty layer
      result.add("");
    }
    else
    {
      for(SLayer l : layers)
      {
        result.add(l.getName());
      }
    }
    
    return result;
  }
}
