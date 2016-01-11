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
import annis.ql.parser.QueryData;
import annis.sqlgen.CountSqlGenerator;
import annis.utils.Utils;
import java.util.List;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.BeansException;
import org.springframework.context.support.GenericXmlApplicationContext;

/**
 *
 * @author thomas
 */
public class QueryToSQL
{
  
  private static final Logger log = LoggerFactory.getLogger(QueryToSQL.class);

  private QueryDao queryDao;
  private CountSqlGenerator countGen;

  public QueryToSQL()
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
    }
    catch (BeansException | IllegalStateException ex)
    {
      log.error(ex.getMessage(), ex);
    }
  }
  
  public String serializeQuery(QueryData queryData, List<String> corporaNames)
  {
    if(queryDao == null || countGen == null)
    {
      return null;
    }
    else
    {
      List<Long> corpusIDs = queryDao.mapCorpusNamesToIds(corporaNames);
      queryData.setCorpusList(corpusIDs);
      queryData.setDocuments(null);
      return countGen.toSql(queryData);
    }
  }
}
