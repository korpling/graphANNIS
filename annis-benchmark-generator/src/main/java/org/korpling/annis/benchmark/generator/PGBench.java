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
package org.korpling.annis.benchmark.generator;

import annis.AnnisXmlContextHelper;
import annis.dao.QueryDao;
import annis.ql.parser.AnnisParserAntlr;
import annis.ql.parser.QueryData;
import annis.sqlgen.CountSqlGenerator;
import annis.utils.Utils;
import com.google.common.io.Files;
import java.io.File;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.LinkedList;
import java.util.List;
import java.util.logging.Level;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.BeansException;
import org.springframework.context.support.GenericXmlApplicationContext;

/**
 *
 * @author thomas
 */
public class PGBench
{

  private static final Logger log = LoggerFactory.getLogger(PGBench.class);

  private QueryDao queryDao;

  private CountSqlGenerator countGen;
  
  private AnnisParserAntlr parser;

  private void initSpring()
  {
    try
    {
      String path = Utils.getAnnisFile(
        "conf/spring/Dao.xml").getAbsolutePath();
      GenericXmlApplicationContext ctx = new GenericXmlApplicationContext();
      ctx.setValidating(false);

      AnnisXmlContextHelper.prepareContext(ctx);

      ctx.load("file:" + path);
      ctx.refresh();

      this.queryDao = ctx.getBean("queryDao", QueryDao.class);
      this.countGen = ctx.getBean("countSqlGenerator", CountSqlGenerator.class);
      this.parser = ctx.getBean("annisParserAntlr", AnnisParserAntlr.class);
    }
    catch (BeansException | IllegalStateException ex)
    {
      log.error(ex.getMessage(), ex);
    }
  }

  public String export(QueryData queryData, List<String> corporaNames)
  {
    List<Long> corpusIDs = queryDao.mapCorpusNamesToIds(corporaNames);
    queryData.setCorpusList(corpusIDs);
    queryData.setDocuments(null);
    return countGen.toSql(queryData).replace('\n', ' ');

  }

  public QueryDao getQueryDao()
  {
    if (queryDao == null)
    {
      initSpring();
    }
    return queryDao;
  }

  public CountSqlGenerator getCountGen()
  {
    if (countGen == null)
    {
      initSpring();
    }
    return countGen;
  }

  public AnnisParserAntlr getParser()
  {
    if (parser == null)
    {
      initSpring();
    }
    return parser;
  }
  
  

  /**
   * @param args the command line arguments
   */
  public static void main(String[] args)
  {
    if(args.length != 2)
    {
      log.error("Invalid arguments: should be <input directory> <output file>");
      System.exit(-1);
    }
    PGBench pg = new PGBench();
    List<Query> queries = QuerySetPersistance.loadQuerySet(new File(args[0]));
    
    List<String> sql = new LinkedList<>();
    
    for(Query q : queries)
    {
      QueryData queryData = pg.getParser().parse(q.getAql(), null);
      sql.add(pg.export(queryData, new LinkedList<>(q.getCorpora())));
    }
    
    try
    {
      Files.asCharSink(new File(args[1]), StandardCharsets.UTF_8).writeLines(sql);
    }
    catch (IOException ex)
    {
      log.error(null, ex);
    }
  }

}
