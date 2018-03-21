/*
 * Copyright 2018 Thomas Krause.
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
package org.corpus_tools.graphannis.api;

/**
 *
 * @author thomas
 */
public enum LogLevel {
  Off(0), Error(1), Warn(2), Info(3), Debug(4), Trace(5);
  private int raw;

  LogLevel(int raw)
  {
    this.raw = raw;
  }

  public int getRaw()
  {
    return this.raw;
  }
  
}
