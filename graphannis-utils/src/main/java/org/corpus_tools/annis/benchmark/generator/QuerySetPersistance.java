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
package org.corpus_tools.annis.benchmark.generator;

import com.google.common.base.Joiner;
import com.google.common.base.Preconditions;
import com.google.common.base.Splitter;
import com.google.common.io.Files;
import java.io.File;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Collection;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicInteger;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 *
 * @author thomas
 */
public class QuerySetPersistance
{
  
  private static final Logger log = LoggerFactory.getLogger(QuerySetPersistance.class);

  public static List<Query> loadQuerySet(File dir)
  {
    ArrayList<Query> qs = new ArrayList<>();
    
    // find all ".aql" files
    File[] aqlFiles = dir.listFiles((File dir1, String name) ->
    {
      return name.endsWith(".aql");
    });
    for(File f : aqlFiles)
    {
      String name = f.getName().substring(0, f.getName().length() - ".aql".length());
      Query q = loadQuery(dir, name);
      qs.add(q);
    }
    
    return qs;
  }
  
  private static Query loadQuery(File parentDir, String name)
  {
    Preconditions.checkNotNull(parentDir);
    Preconditions.checkNotNull(name);
    Preconditions.checkArgument(parentDir.isDirectory());
    
    Query q = new Query();
    q.setName(name);
    
    File fAql = new File(parentDir, name + ".aql");
    if(fAql.isFile())
    {
      try
      {
        q.setAql(Files.asCharSource(fAql, StandardCharsets.UTF_8).read());
      }
      catch(IOException ex)
      {
        log.error(null, ex);
      }
    }
    
    File fJson = new File(parentDir, name + ".json");
    if(fJson.isFile())
    {
      try
      {
        q.setJson(Files.asCharSource(fJson, StandardCharsets.UTF_8).read());
      }
      catch(IOException ex)
      {
        log.error(null, ex);
      }
    }
    
    File fCount = new File(parentDir, name + ".count");
    if(fCount.isFile())
    {
      try
      {
        String raw = Files.asCharSource(fCount, StandardCharsets.UTF_8).read();
        q.setCount(Optional.of(Long.parseLong(raw.trim())));
      }
      catch(IOException ex)
      {
        log.error(null, ex);
      }
    }
    
    File fTime = new File(parentDir, name + ".time");
    if(fTime.isFile())
    {
      try
      {
        String raw = Files.asCharSource(fTime, StandardCharsets.UTF_8).read();
        q.setExecutionTime(Optional.of(Long.parseLong(raw.trim())));
      }
      catch(IOException ex)
      {
        log.error(null, ex);
      }
    }
    
    
    File fCorpora = new File(parentDir, name + ".corpora");
    if(fCorpora.isFile())
    {
      try
      {
        String raw = Files.asCharSource(fCorpora, StandardCharsets.UTF_8).read();
        q.setCorpora(new LinkedHashSet<>(Splitter.on(",").omitEmptyStrings().trimResults().splitToList(raw)));
      }
      catch(IOException ex)
      {
        log.error(null, ex);
      }
    }
    
    return q;
    
  }
  
  public static int writeQuerySet(File dir, Collection<Query> queries)
  {
    final AtomicInteger success = new AtomicInteger(0);
    Preconditions.checkArgument(dir.isDirectory());
    queries.stream().
      forEach((q) ->
    {
      try
      {
        writeQuery(dir, q);
        success.incrementAndGet();
      }
      catch(IOException ex)
      {
        log.error(null, ex);
      }
    });
    return success.get();
  }

  private static void writeQuery(File parentDir, Query q)
    throws IOException
  {
    Preconditions.checkNotNull(q.getName());
    Preconditions.checkNotNull(q.getAql());
    
    String name = q.getName();
    File fAQL = new File(parentDir, name + ".aql");
    Files.write(q.getAql() + "\n", fAQL, StandardCharsets.UTF_8);
    
    if(q.getJson() != null)
    {
      File fJSON = new File(parentDir, name + ".json");
      Files.write(q.getJson(), fJSON, StandardCharsets.UTF_8); 
    }
    if(q.getCount().isPresent())
    {
      File fCount = new File(parentDir, name + ".count");
      Files.write("" + q.getCount().get(), fCount, StandardCharsets.UTF_8); 
    }
    if(q.getExecutionTime().isPresent())
    {
      File fTime = new File(parentDir, name + ".time");
      Files.write("" + q.getExecutionTime().get(), fTime, StandardCharsets.UTF_8); 
    }
    if(q.getCorpora() != null && !q.getCorpora().isEmpty())
    {
      File fCorpora = new File(parentDir, name + ".corpora");
      Files.write(Joiner.on(",").join(q.getCorpora()), fCorpora, StandardCharsets.UTF_8); 
    }

  }
}
