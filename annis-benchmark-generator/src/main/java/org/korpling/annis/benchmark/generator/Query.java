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

import java.util.LinkedHashSet;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;

/**
 *
 * @author thomas
 */
public class Query
{
  
  private String name;
  
  private String aql;

  private Set<String> corpora = new LinkedHashSet<>();

  private Optional<Long> count = Optional.empty();

  private Optional<Long> executionTime = Optional.empty();

  private String json;
  
  private String sql;

  public String getJson()
  {
    return json;
  }

  public void setJson(String json)
  {
    this.json = json;
  }

  
  public String getAql()
  {
    return aql;
  }

  public void setAql(String aql)
  {
    this.aql = aql;
  }

  public Set<String> getCorpora()
  {
    return corpora;
  }

  public void setCorpora(Set<String> corpora)
  {
    this.corpora = corpora;
  }

  

  public Optional<Long> getCount()
  {
    return count;
  }

  public void setCount(Optional<Long> count)
  {
    this.count = count == null ? Optional.empty() : count;
  }

  public Optional<Long> getExecutionTime()
  {
    return executionTime;
  }

  public void setExecutionTime(Optional<Long> executionTime)
  {
    this.executionTime = executionTime == null ? Optional.empty() : executionTime;
  }

  public String getName()
  {
    return name;
  }

  public void setName(String name)
  {
    this.name = name;
  }

  public String getSql()
  {
    return sql;
  }

  public void setSql(String sql)
  {
    this.sql = sql;
  }

  @Override
  public int hashCode()
  {
    int hash = 7;
    hash = 37 * hash + Objects.hashCode(this.name);
    hash = 37 * hash + Objects.hashCode(this.aql);
    hash = 37 * hash + Objects.hashCode(this.corpora);
    hash = 37 * hash + Objects.hashCode(this.count);
    hash = 37 * hash + Objects.hashCode(this.executionTime);
    hash = 37 * hash + Objects.hashCode(this.json);
    hash = 37 * hash + Objects.hashCode(this.sql);
    return hash;
  }

  @Override
  public boolean equals(Object obj)
  {
    if (this == obj)
    {
      return true;
    }
    if (obj == null)
    {
      return false;
    }
    if (getClass() != obj.getClass())
    {
      return false;
    }
    final Query other = (Query) obj;
    if (!Objects.equals(this.name, other.name))
    {
      return false;
    }
    if (!Objects.equals(this.aql, other.aql))
    {
      return false;
    }
    if (!Objects.equals(this.json, other.json))
    {
      return false;
    }
    if (!Objects.equals(this.sql, other.sql))
    {
      return false;
    }
    if (!Objects.equals(this.corpora, other.corpora))
    {
      return false;
    }
    if (!Objects.equals(this.count, other.count))
    {
      return false;
    }
    if (!Objects.equals(this.executionTime, other.executionTime))
    {
      return false;
    }
    return true;
  }

  

}
