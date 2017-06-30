/*
   Copyright 2017 Thomas Krause <thomaskrause@posteo.de>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#pragma once

#include <annis/iterators.h>  // for AnnoIt
#include <stdint.h>           // for int64_t
#include <set>                // for set
#include <string>             // for string
#include <unordered_set>      // for unordered_set

#include <boost/optional.hpp>

namespace annis { struct Annotation; }
namespace annis { struct AnnotationKey; }


namespace annis
{

class EstimatedSearch : public AnnoIt
{
public:
  virtual std::int64_t guessMaxCount() const {return -1;}

  virtual std::string debugString() const {return "";}


  /**
   * @brief Set a constant annotation value that is returned in a match instead of the actual matched annotation.
   *
   * The node ID part of the match is still the actual match, but the annotation is replaced by this constant value.
   * This can be useful when searching for nodes (e.g. token) after a specific criterium but the result should
   * include the node ID but not the specific annotation that what searched for. Otherwise matches could be
   * regarded as different because their annotation is the differet.
   *
   * @param constAnno
   */
  void setConstAnnoValue(boost::optional<Annotation> constAnno)
  {
    _constAnno = constAnno;
  }

  boost::optional<Annotation> getConstAnnoValue()
  {
    return _constAnno;
  }
private:
  boost::optional<Annotation> _constAnno;
};


} // end namespace annis
