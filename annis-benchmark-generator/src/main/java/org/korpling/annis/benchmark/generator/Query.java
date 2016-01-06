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

import java.util.Objects;
import java.util.Optional;

/**
 *
 * @author thomas
 */
public class Query
{

  private String aql;

  private String corpus;

  private Optional<Long> count = Optional.empty();

  private Optional<Long> executionTime = Optional.empty();

  public String getAql()
  {
    return aql;
  }

  public void setAql(String aql)
  {
    this.aql = aql;
  }

  public String getCorpus()
  {
    return corpus;
  }

  public void setCorpus(String corpus)
  {
    this.corpus = corpus;
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

  @Override
  public int hashCode()
  {
    int hash = 5;
    hash = 19 * hash + Objects.hashCode(this.aql);
    hash = 19 * hash + Objects.hashCode(this.corpus);
    hash = 19 * hash + Objects.hashCode(this.count);
    hash = 19 * hash + Objects.hashCode(this.executionTime);
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
    if (!Objects.equals(this.aql, other.aql))
    {
      return false;
    }
    if (!Objects.equals(this.corpus, other.corpus))
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
