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
package org.korpling.annis.benchmark.generator;

import annis.model.QueryNode;
import annis.ql.parser.QueryData;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.fasterxml.jackson.module.jaxb.JaxbAnnotationModule;
import java.util.Arrays;
import java.util.List;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 *
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class QueryToJSON
{
  
  private static final Logger log = LoggerFactory.getLogger(QueryToJSON.class);
  
  private static final JsonNodeFactory factory = new JsonNodeFactory(true);
  
  private static final JaxbAnnotationModule jaxbModule = new JaxbAnnotationModule();
  
  
  /**
   * This will serialize the query part of the {@link QueryData} to JSON.
   * @param queryData
   * @return 
   */
  public static String serializeQuery(QueryData queryData)
  {
    return queryAsJSON(queryData).toString();
  }
  
  public static ObjectNode queryAsJSON(QueryData queryData)
  {
    ObjectNode root = factory.objectNode();
    ObjectMapper mapper = new ObjectMapper();
    mapper.registerModule(jaxbModule);
    mapper.setSerializationInclusion(JsonInclude.Include.NON_EMPTY);
    
    if(queryData.getAlternatives() != null && !queryData.getAlternatives().isEmpty())
    {
      ArrayNode alternatives = root.putArray("alternatives");
      for(List<QueryNode> alt : queryData.getAlternatives())
      {
        ObjectNode altNode = alternatives.addObject();
       
        ObjectNode nodes = altNode.putObject("nodes");
        
        for(QueryNode n : alt)
        {
          JsonNode nodeObject = mapper.valueToTree(n);
          // manually remove some internal fields
          if(nodeObject instanceof ObjectNode)
          {
            ((ObjectNode) nodeObject).remove(Arrays.asList("partOfEdge", 
              "artificial", "alternativeNumber", "parseLocation"));
          }
          nodes.set("" + n.getId(), nodeObject);
          
          
        }
      }
    }
    
    return root;
  }
}
