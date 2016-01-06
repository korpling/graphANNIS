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

import com.google.common.io.LineProcessor;
import java.io.IOException;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 *
 * @author thomas
 */
public class QueryLogParser implements LineProcessor<QuerySet>
{

  private final QuerySet queries = new QuerySet();

  private StringBuilder currentAQL;

  private static final Pattern COMPLETE_LINE = Pattern.compile(
    "^[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9](.*)function: COUNT, query: (?<query>.*), corpus: \\[(?<corpus>[^\\]]+)\\], runtime: (?<time>[0-9]+) ms$");

  private static final Pattern INCOMPLETE_START = Pattern.compile(
    "^[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9](.*)function: COUNT, query: (?<query>.*)$");

  private static final Pattern INCOMPLETE_END = Pattern.compile(
    "(?<query>.*)corpus: \\[(?<corpus>[^\\]]+)\\], runtime: (?<time>[0-9]+) ms$");

  @Override
  public boolean processLine(String line) throws IOException
  {
    if (currentAQL == null)
    {
      Matcher mComplete = COMPLETE_LINE.matcher(line);
      Matcher mStart = INCOMPLETE_START.matcher(line);
      if (COMPLETE_LINE.matcher(line).matches())
      {
        Query q = new Query();
        q.setAql(mComplete.group("query"));
        q.setCorpus(mComplete.group("corpus"));
        q.setExecutionTime(Optional.of(Long.parseLong(mComplete.group("time"))));
        queries.add(q);
      }
      else if (mStart.matches())
      {
        currentAQL = new StringBuilder();
        currentAQL.append(mStart.group("query"));
      }
    }
    else
    {
      // we have a query we need to complete
      Matcher mEnd = INCOMPLETE_END.matcher(line);
      if (mEnd.matches())
      {
        // query is finished
        currentAQL.append(mEnd.group("query"));
        Query q = new Query();
        q.setAql(currentAQL.toString());
        q.setCorpus(mEnd.group("corpus"));
        q.setExecutionTime(Optional.of(Long.parseLong(mEnd.group("time"))));
        queries.add(q);
        
        currentAQL = null;
      }
      else
      {
        currentAQL.append(line);
      }
    }
    return true;
  }

  @Override
  public QuerySet getResult()
  {
    return queries;
  }

}
