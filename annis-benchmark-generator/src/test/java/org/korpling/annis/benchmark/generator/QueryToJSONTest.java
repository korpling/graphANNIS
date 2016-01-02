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
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import com.fasterxml.jackson.databind.node.ObjectNode;
import java.io.IOException;
import java.util.ArrayList;
import java.util.LinkedList;
import java.util.List;
import org.junit.After;
import org.junit.AfterClass;
import org.junit.Before;
import org.junit.BeforeClass;
import org.junit.Test;
import static org.junit.Assert.*;

/**
 *
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class QueryToJSONTest
{
  
  private final JsonNodeFactory nodeFactory;
  
  public QueryToJSONTest()
  {
    nodeFactory = new JsonNodeFactory(true);
  }
  
  @BeforeClass
  public static void setUpClass()
  {
  }
  
  @AfterClass
  public static void tearDownClass()
  {
  }
  
  @Before
  public void setUp()
  {
  }
  
  @After
  public void tearDown()
  {
  }

  /**
   * Test to serialize an empty query data.
   */
  @Test
  public void testSerializeQuery()
  {
    System.out.println("serializeQuery");
    
    
    
    QueryData queryData = new QueryData();
    String expResult = "{}";
    String result = QueryToJSON.serializeQuery(queryData);
    assertEquals(expResult, result);
  }

  /**
   * Test to map an empty query to JSON.
   */
  @Test
  public void testQueryAsJSON()
  {
    System.out.println("queryAsJSON");
    QueryData queryData = new QueryData();
    
    ObjectNode expResult = nodeFactory.objectNode();
    
    ObjectNode result = QueryToJSON.queryAsJSON(queryData);
    assertEquals(expResult, result);
  }
  
  /**
   * Test if query only has one alternative an one node if embeddeding the node has worked.
   */
  @Test
  public void testOneNode() throws IOException
  {
    System.out.println("oneNode");
    
    QueryData queryData = new QueryData();
    List<QueryNode> alt = new ArrayList<>();
    
    QueryNode n1 = new QueryNode(1);
    n1.setVariable("1");
    n1.setToken(true);
    n1.setArtificial(false);
    alt.add(n1);
    
    queryData.addAlternative(alt);
    
    ObjectMapper mapper = new ObjectMapper();
    mapper.enable(DeserializationFeature.USE_LONG_FOR_INTS);
   
    ObjectNode expResult = (ObjectNode) mapper.readTree("{"
      + "\"alternatives\" : "
      + " ["
      + "  {\"nodes\" : {\"1\": {\"id\": 1, \"root\": false, \"token\":true, \"variable\": \"1\"}}}"
      + " ]"
      + "}");
    
    ObjectNode result = QueryToJSON.queryAsJSON(queryData);
    
    assertEquals(expResult, result);
    
  }
  
}
