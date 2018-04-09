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
package org.corpus_tools.annis.benchmark.generator;

import annis.model.QueryNode;

import org.corpus_tools.annis.ql.model.Dominance;
import org.corpus_tools.annis.ql.model.Inclusion;
import org.corpus_tools.annis.ql.model.Overlap;
import org.corpus_tools.annis.ql.model.PointingRelation;
import org.corpus_tools.annis.ql.model.Precedence;
import org.corpus_tools.annis.ql.parser.QueryData;

import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import com.fasterxml.jackson.databind.node.ObjectNode;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import org.junit.After;
import org.junit.AfterClass;
import org.junit.Before;
import org.junit.BeforeClass;
import org.junit.Test;
import org.corpus_tools.graphannis.QueryToJSON;

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
    String result = QueryToJSON.serializeQuery(queryData.getAlternatives(), queryData.getMetaData());
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
    
    ObjectNode result = QueryToJSON.queryAsJSON(queryData.getAlternatives(), queryData.getMetaData());
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
      + "  {\"nodes\" : {\"1\": {\"id\": 1, \"nodeAnnotations\":[], \"root\": false, \"token\":true, \"variable\": \"1\"}},"
      + "  \"joins\":[]}"
      + " ]"
      + "}");
    
    ObjectNode result = QueryToJSON.queryAsJSON(queryData.getAlternatives(), queryData.getMetaData());
    
    assertEquals(expResult, result);
    
  }
  
  /**
   * Test if query only has one alternative an one node if embeddeding the node has worked.
   */
  @Test
  public void testTwoJoins() throws IOException
  {
    System.out.println("twoJoins");
    
    QueryData queryData = new QueryData();
    List<QueryNode> alt = new ArrayList<>();
    
    QueryNode n1 = new QueryNode(1);
    n1.setVariable("1");
    
    QueryNode n2 = new QueryNode(2);
    n2.setVariable("2");
    
    QueryNode n3 = new QueryNode(3);
    n3.setVariable("3");
    
    Precedence precedenceJoin = new Precedence(n2, 2, 10);
    n1.addOutgoingJoin(precedenceJoin);
    
    Overlap overlapJoin = new Overlap(n3);
    n2.addOutgoingJoin(overlapJoin);
    
    Inclusion inclusionJoin = new Inclusion(n3);
    n1.addOutgoingJoin(inclusionJoin);
    
    Dominance dominanceJoin = new Dominance(n1, "cat", 3, 5);
    n2.addOutgoingJoin(dominanceJoin);
    
    PointingRelation pointingJoin = new PointingRelation(n1, "dep", 1, 2);
    n3.addOutgoingJoin(pointingJoin);
    
    alt.add(n1);
    alt.add(n2);
    alt.add(n3);
    
    queryData.addAlternative(alt);
    
    ObjectMapper mapper = new ObjectMapper();
    mapper.enable(DeserializationFeature.USE_LONG_FOR_INTS);
   
    String expJson = "{"
      + "\"alternatives\" : "
      + " ["
      + "  {"
      + "    \"nodes\" : {"
      + "       \"1\": {\"id\": 1, \"nodeAnnotations\":[], \"root\": false, \"token\":false, \"variable\": \"1\"}, "
      + "       \"2\": {\"id\": 2, \"nodeAnnotations\":[], \"root\": false, \"token\":false, \"variable\": \"2\"}, "
      + "       \"3\": {\"id\": 3, \"nodeAnnotations\":[], \"root\": false, \"token\":false, \"variable\": \"3\"} "
      + "    },"
      + "    \"joins\": ["
      + "      {\"op\": \"Precedence\", \"minDistance\":2, \"maxDistance\":10, \"left\": 1, \"right\": 2},"
      + "      {\"op\": \"Inclusion\", \"left\": 1, \"right\": 3},"
      + "      {\"op\": \"Overlap\", \"left\": 2, \"right\": 3},"
      + "      {\"op\": \"Dominance\", \"name\": \"cat\", \"minDistance\":3, \"maxDistance\":5, \"left\": 2, \"right\": 1},"
      + "      {\"op\": \"Pointing\", \"name\": \"dep\", \"minDistance\":1, \"maxDistance\":2, \"left\": 3, \"right\": 1}"
      + "    ]"
      + "  }"
      + " ]"
      + "}";
    ObjectNode expResult = (ObjectNode) mapper.readTree(expJson);
    
    ObjectNode result = QueryToJSON.queryAsJSON(queryData.getAlternatives(), queryData.getMetaData());
    
    assertEquals(expResult, result);
    
  }
  
}
