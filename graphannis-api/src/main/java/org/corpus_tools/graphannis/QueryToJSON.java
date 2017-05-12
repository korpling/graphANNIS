/*
 * Copyright 2016 Thomas Krause <thomaskrause@posteo.de>.
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
import java.util.List;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.fasterxml.jackson.module.jaxb.JaxbAnnotationModule;

import annis.exceptions.AnnisQLSyntaxException;
import annis.model.Join;
import annis.model.QueryAnnotation;
import annis.model.QueryNode;
import annis.model.QueryNode.TextMatching;
import annis.sqlgen.model.CommonAncestor;
import annis.sqlgen.model.Dominance;
import annis.sqlgen.model.Inclusion;
import annis.sqlgen.model.LeftDominance;
import annis.sqlgen.model.Overlap;
import annis.sqlgen.model.PointingRelation;
import annis.sqlgen.model.Precedence;
import annis.sqlgen.model.RightDominance;
import annis.sqlgen.model.SameSpan;
import annis.sqlgen.model.Sibling;
import java.util.LinkedList;
import org.corpus_tools.annis.ql.parser.AnnisParserAntlr;
import org.corpus_tools.annis.ql.parser.QueryData;

/**
 *
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class QueryToJSON
{

  private static final JsonNodeFactory factory = new JsonNodeFactory(true);

  private static final JaxbAnnotationModule jaxbModule = new JaxbAnnotationModule();
  
  public static String aqlToJSON(String aql)
  {
    AnnisParserAntlr parser = new AnnisParserAntlr();
    
    QueryData qd = parser.parse(aql, new LinkedList<>());
    return serializeQuery(qd.getAlternatives(), qd.getMetaData());
  }
  
  /**
   * This will serialize the query part of the {@link QueryData} to JSON.
   *
   * @param query
   * @param metaData
   * @return
   */
  public static String serializeQuery(List<List<QueryNode>> query, 
      List<QueryAnnotation> metaData)
  {
    return queryAsJSON(query, metaData).toString();
  }

  public static ObjectNode queryAsJSON(List<List<QueryNode>> query, 
      List<QueryAnnotation> metaData)
  {
    ObjectNode root = factory.objectNode();
    ObjectMapper mapper = new ObjectMapper();
    mapper.registerModule(jaxbModule);
    mapper.setSerializationInclusion(JsonInclude.Include.NON_EMPTY);
    
    if (query != null && !query.isEmpty())
    {
      ArrayNode alternatives = root.putArray("alternatives");
      for (List<QueryNode> alt : query)
      {
        ObjectNode altNode = alternatives.addObject();

        ObjectNode nodes = altNode.putObject("nodes");
        ArrayNode joinObject = altNode.putArray("joins");
        
        // map each node
        for (QueryNode n : alt)
        {
          if(n.getSpanTextMatching() == QueryNode.TextMatching.EXACT_NOT_EQUAL || n.getSpanTextMatching() == QueryNode.TextMatching.REGEXP_NOT_EQUAL)
          {
            throw new AnnisQLSyntaxException("negation not supported yet");
          }
          
          if(n.isRoot())
          {
            throw new AnnisQLSyntaxException("\"root\" operator not supported yet");
          }
          
          if(n.getArity() != null)
          {
            throw new AnnisQLSyntaxException("\"arity\" operator not supported yet");
          }
          
          if(n.getTokenArity() != null)
          {
            throw new AnnisQLSyntaxException("\"tokenarity\" operator not supported yet");
          }
          
          for(QueryAnnotation anno : n.getNodeAnnotations())
          {
            if (anno.getTextMatching() == QueryNode.TextMatching.EXACT_NOT_EQUAL 
              || anno.getTextMatching()== QueryNode.TextMatching.REGEXP_NOT_EQUAL)
            {
              throw new AnnisQLSyntaxException(
                "negation not supported yet");
            }
          }
          for(QueryAnnotation anno : n.getEdgeAnnotations())
          {
            if (anno.getTextMatching() == QueryNode.TextMatching.EXACT_NOT_EQUAL 
              || anno.getTextMatching()== QueryNode.TextMatching.REGEXP_NOT_EQUAL)
            {
              throw new AnnisQLSyntaxException(
                "negation not supported yet");
            }
          }
          JsonNode nodeObject = mapper.valueToTree(n);
          // manually remove some internal fields
          if (nodeObject instanceof ObjectNode)
          {
            ((ObjectNode) nodeObject).remove(Arrays.asList("partOfEdge",
              "artificial", "alternativeNumber", "parseLocation"));
          }
          nodes.set("" + n.getId(), nodeObject);

          // map outgoing joins
          for (Join aqlJoin : n.getOutgoingJoins())
          {
            ObjectNode j = joinObject.addObject();
            mapJoin(aqlJoin, n, j, mapper);
          }
        } // end for each node of a single alternative
        
        // also add the meta-data as a special node and connect it with a SubPartOfCorpus join
        if(metaData != null && !metaData.isEmpty() && !alt.isEmpty())
        {
          altNode.set("meta", mapper.valueToTree(metaData));
        }
        
      } // end for each alternative
    } // end if query not empty

    return root;
  }

  private static void mapJoin(Join join, QueryNode source, ObjectNode node,
    ObjectMapper mapper)
  {
    // TODO: more join types and features
    if(join instanceof CommonAncestor
        || join instanceof LeftDominance
        || join instanceof RightDominance
        || join instanceof Sibling)
    {
      // these are specializations of Dominance we explicitly don't support yet
      throw new AnnisQLSyntaxException(
          "This join type can't be mapped yet: " + join.toAQLFragment(source));
      
    }
    else if (join instanceof Dominance)
    {
      node.put("op", "Dominance");
      Dominance dom = (Dominance) join;
      node.put("name", dom.getName() == null ? "" : dom.getName());
      node.put("minDistance", (long) dom.getMinDistance());
      node.put("maxDistance", (long) dom.getMaxDistance());
      if (!dom.getEdgeAnnotations().isEmpty())
      {
        for(QueryAnnotation anno : dom.getEdgeAnnotations())
        {
          if(anno.getTextMatching() != TextMatching.EXACT_EQUAL)
          {
            throw new AnnisQLSyntaxException(
                "Only non-regex and non-negated edge annotations are supported yet");
          }
        }
        
        JsonNode edgeAnnos = mapper.valueToTree(dom.getEdgeAnnotations());
        node.set("edgeAnnotations", edgeAnnos);
      }
    }
    else if (join instanceof PointingRelation)
    {
      node.put("op", "Pointing");
      PointingRelation pointing = (PointingRelation) join;
      node.put("name", pointing.getName() == null ? "" : pointing.getName());
      node.put("minDistance", (long) pointing.getMinDistance());
      node.put("maxDistance", (long) pointing.getMaxDistance());
      if (!pointing.getEdgeAnnotations().isEmpty())
      {
        for(QueryAnnotation anno : pointing.getEdgeAnnotations())
        {
          if(anno.getTextMatching() != TextMatching.EXACT_EQUAL)
          {
            throw new AnnisQLSyntaxException(
                "Only non-regex and non-negated edge annotations are supported yet");
          }
        }
        
        JsonNode edgeAnnos = mapper.valueToTree(pointing.getEdgeAnnotations());
        node.set("edgeAnnotations", edgeAnnos);
      }
    }
    else if (join instanceof Precedence)
    {
      node.put("op", "Precedence");
      Precedence prec = (Precedence) join;
      node.put("minDistance", (long) prec.getMinDistance());
      node.put("maxDistance", (long) prec.getMaxDistance());
    }
    else if (join instanceof Overlap)
    {
      node.put("op", "Overlap");
    }
    else if (join instanceof Inclusion)
    {
      node.put("op", "Inclusion");
    }
    else if(join instanceof SameSpan)
    {
      node.put("op", "IdenticalCoverage");
    }
    else
    {
      throw new AnnisQLSyntaxException(
        "This join type can't be mapped yet: " + join.toAQLFragment(source));
    }

    node.put("left", (long) source.getId());
    node.put("right", (long) join.getTarget().getId());
  }
}
