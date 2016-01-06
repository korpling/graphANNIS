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

import com.google.common.collect.HashMultiset;
import com.google.common.collect.Multiset;
import java.util.Collection;
import java.util.LinkedList;
import java.util.List;

/**
 *
 * @author thomas
 */
public class QuerySet
{
  private final Multiset<Query> queries = HashMultiset.create();
  
  public void add(Query q)
  {
    queries.add(q);
  }
  
  public void addAll(Collection<Query> newQueries)
  {
    queries.addAll(newQueries);
  }
  
  public void clear()
  {
    queries.clear();
  }
  
  public List<Query> getAll()
  {
    return new LinkedList<>(queries);
  }
  
  public int size()
  {
    return queries.size();
  }
}
