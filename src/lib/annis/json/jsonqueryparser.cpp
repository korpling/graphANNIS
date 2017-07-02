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

#include "jsonqueryparser.h"
#include <annis/annosearch/exactannokeysearch.h>    // for ExactAnnoKeySearch
#include <annis/annosearch/exactannovaluesearch.h>  // for ExactAnnoValueSearch
#include <annis/annosearch/regexannosearch.h>       // for RegexAnnoSearch
#include <annis/operators/dominance.h>              // for Dominance
#include <annis/operators/identicalcoverage.h>      // for IdenticalCoverage
#include <annis/operators/identicalnode.h>
#include <annis/operators/inclusion.h>              // for Inclusion
#include <annis/operators/overlap.h>                // for Overlap
#include <annis/operators/pointing.h>               // for Pointing
#include <annis/operators/precedence.h>             // for Precedence
#include <annis/operators/partofsubcorpus.h>
#include <assert.h>                                 // for assert
#include <re2/re2.h>                                // for RE2
#include <limits>                                   // for numeric_limits
#include <map>                                      // for _Rb_tree_const_it...
#include <utility>                                  // for pair
#include "annis/db.h"                               // for DB
#include "annis/json/json.h"                        // for Value, ValueConst...
#include "annis/query/query.h"                            // for Query
#include "annis/queryconfig.h"                      // for QueryConfig
#include "annis/stringstorage.h"                    // for StringStorage
#include "annis/types.h"                          // for Edge, GraphStatistic

#include <boost/optional.hpp>

#include <functional>

using namespace annis;

JSONQueryParser::JSONQueryParser()
{
}
std::shared_ptr<Query> JSONQueryParser::parse(DB &db, std::istream& jsonStream, const QueryConfig config)
{
  return parse(db, db.f_getGraphStorage, db.f_getAllGraphStorages, jsonStream, config);
}

std::shared_ptr<Query> JSONQueryParser::parse(const DB& db, DB::GetGSFuncT getGraphStorageFunc,
                                              DB::GetAllGSFuncT getAllGraphStorageFunc,
                                              std::istream& jsonStream, const QueryConfig config)
{
  std::vector<std::shared_ptr<SingleAlternativeQuery>> result;

  // parse root as value
  Json::Value root;
  jsonStream >> root;

  // iterate over all alternatives
  const auto& alternatives = root["alternatives"];
  for(const auto& alt : alternatives)
  {
    std::shared_ptr<SingleAlternativeQuery> q = std::make_shared<SingleAlternativeQuery>(db, config);


    // add all nodes
    const auto& nodes = alt["nodes"];

    std::map<std::uint64_t, size_t> nodeIdToPos;
    boost::optional<size_t> firstNodePos;
    for (auto it = nodes.begin(); it != nodes.end(); it++)
    {
      auto& n = *it;
      size_t pos = parseNode(db, n, getGraphStorageFunc, q);
      nodeIdToPos[std::stoull(it.name())] = pos;
      if(!firstNodePos)
      {
        firstNodePos = pos;
      }
    }

    // add all joins
    const auto& joins = alt["joins"];
    for (auto it = joins.begin(); it != joins.end(); it++)
    {
      parseJoin(db, getGraphStorageFunc, getAllGraphStorageFunc, *it, q, nodeIdToPos);
    }

    // add all meta-data
    const auto& meta = alt["meta"];
    boost::optional<size_t> firstMetaIdx = boost::none;
    for (auto it = meta.begin(); it != meta.end(); it++)
    {
      auto& m = *it;

      // add an artificial node that describes the document/corpus node
      size_t metaNodeIdx = addNodeAnnotation(db, q, optStr(m["namespace"]),
            optStr(m["name"]), optStr(m["value"]),
            optStr(m["textMatching"]), true);

      if(firstMetaIdx)
      {
        // avoid nested loops by joining additional meta nodes with a "identical node"
        q->addOperator(std::make_shared<IdenticalNode>(db), metaNodeIdx, *firstMetaIdx);

      }
      else
      {
        firstMetaIdx = metaNodeIdx;
        // add a special join to the first node of the query
        q->addOperator(std::make_shared<PartOfSubCorpus>(getGraphStorageFunc, db.strings, 1),
          metaNodeIdx, *firstNodePos);

      }
    }

    result.push_back(q);

  } // end for each alternative
  return std::make_shared<Query>(result);
}

std::shared_ptr<Query> JSONQueryParser::parseWithUpgradeableLock(DB &db,
                                                                 std::string queryAsJSON, boost::upgrade_lock<DBLoader>& lock,
                                                                 const QueryConfig config)
{
  DB::GetAllGSFuncT allFunc = [&db,&lock](ComponentType type, const std::string &name)
  {
    if(db.allGraphStoragesLoaded(type, name))
    {
      return db.getAllGraphStorages(type, name);
    }
    else
    {
      // loading the graph storages needs a unique lock to ensure nobody else is loading the same GS at the same time
      boost::upgrade_to_unique_lock<DBLoader> uniqueLock(lock);
      return db.getAllGraphStorages(type, name);
    }
  };

  DB::GetGSFuncT func = [&db,&lock](ComponentType type, const std::string &layer, const std::string &name)
  {
    if(db.isGraphStorageLoaded(type, layer, name))
    {
      return db.getGraphStorage(type, layer, name);
    }
    else
    {
      // loading the graph storages needs a unique lock to ensure nobody else is loading the same GS at the same time
      boost::upgrade_to_unique_lock<DBLoader> uniqueLock(lock);
      return db.getGraphStorage(type, layer, name);
    }
  };

  std::stringstream ss;
  ss << queryAsJSON;
  return annis::JSONQueryParser::parse(db, func, allFunc, ss, config);
}

size_t JSONQueryParser::parseNode(const DB& db, const Json::Value node, DB::GetGSFuncT getGraphStorageFunc, std::shared_ptr<SingleAlternativeQuery> q)
{

  // annotation search?
  if (node["nodeAnnotations"].isArray() && node["nodeAnnotations"].size() > 0)
  {
    // get the first one
    auto nodeAnno = node["nodeAnnotations"][0];

    return addNodeAnnotation(db, q, optStr(nodeAnno["namespace"]),
      optStr(nodeAnno["name"]), optStr(nodeAnno["value"]),
      optStr(nodeAnno["textMatching"]));

  }// end if annotation search
  else
  {
    // check for special non-annotation search constructs
    // token search?
    if (node["spannedText"].isString()
        || (node["token"].isBool() && node["token"].asBool()))
    {
      size_t n_pos = addNodeAnnotation(db, q, optStr(annis_ns), optStr(annis_tok),
                                       optStr(node["spannedText"]),
                                       optStr(node["spanTextMatching"]), true);

      // special treatment for explicit searches for token (tok="...)
      if(node["token"].isBool() && node["token"].asBool())
      {
        std::shared_ptr<const ReadableGraphStorage> covGS = getGraphStorageFunc(ComponentType::COVERAGE, annis_ns, "");
        if(covGS)
        {
          q->addFilter(n_pos, [&db, covGS] (const Match& m) -> bool
          {
            return static_cast<bool>(db.nodeAnnos.getAnnotations(m.node, db.getNamespaceStringID(), db.getTokStringID()))
                && covGS->getOutgoingEdges(m.node).empty();
          }, "is_token");
        }
      }

      return n_pos;
    }// end if token has spanned text
    else
    {
      // just search for any node
      return addNodeAnnotation(db, q, optStr(annis_ns), optStr(annis_node_type),
        boost::optional<std::string>("node"), boost::optional<std::string>("EXACT_EQUAL"));
    }
  } // end if special case

}

size_t JSONQueryParser::addNodeAnnotation(const DB& db,
  std::shared_ptr<SingleAlternativeQuery> q,
  boost::optional<std::string> ns,
  boost::optional<std::string> name,
  boost::optional<std::string> value,
  boost::optional<std::string> textMatching,
  bool wrapEmptyAnno)
{

  if (value)
  {
    bool exact = *textMatching == "EXACT_EQUAL";
    bool regex = *textMatching == "REGEXP_EQUAL";
    if(regex)
    {
      if(canReplaceRegex(*value))
      {
        exact = true;
      }
    }
    
    // search for the value
    if (exact)
    {
      // has namespace?
      if (ns)
      {
        return q->addNode(std::make_shared<ExactAnnoValueSearch>(db,
          *ns,
          *name,
          *value),
          wrapEmptyAnno);
      }
      else
      {
        return q->addNode(std::make_shared<ExactAnnoValueSearch>(db,
          *name,
          *value),
          wrapEmptyAnno);
      }
    }
    else if (regex)
    {
      // has namespace?
      if (ns)
      {
        return q->addNode(std::make_shared<RegexAnnoSearch>(db,
          *ns,
          *name,
          *value),
          wrapEmptyAnno);
      }
      else
      {
        return q->addNode(std::make_shared<RegexAnnoSearch>(db,
          *name,
          *value),
          wrapEmptyAnno);
      }
    }

  }// end if has value
  else
  {
    // only search for key
    // has namespace?
    if (ns)
    {
      return q->addNode(std::make_shared<ExactAnnoKeySearch>(db,
        *ns,
        *name),
        wrapEmptyAnno);
    }
    else
    {
      return q->addNode(std::make_shared<ExactAnnoKeySearch>(db,
        *name),
        wrapEmptyAnno);
    }
  }
  // should never happen
  assert(false);
  return 0;
}

void JSONQueryParser::parseJoin(const DB& db,
                                DB::GetGSFuncT getGraphStorageFunc,
                                DB::GetAllGSFuncT getAllGraphStorageFunc,
                                const Json::Value join, std::shared_ptr<SingleAlternativeQuery> q,
  const std::map<std::uint64_t, size_t>& nodeIdToPos)
{
  // get left and right index
  if (join["left"].isUInt64() && join["right"].isUInt64())
  {
    auto leftID = join["left"].asUInt64();
    auto rightID = join["right"].asUInt64();

    auto itLeft = nodeIdToPos.find(leftID);
    auto itRight = nodeIdToPos.find(rightID);

    if (itLeft != nodeIdToPos.end() && itRight != nodeIdToPos.end())
    {

      auto op = join["op"].asString();
      if (op == "Precedence")
      {
        auto minDist = join["minDistance"].asUInt();
        auto maxDist = join["maxDistance"].asUInt();
        if(join["segmentation-name"].isString() && join["segmentation-name"].asString() != "")
        {
          q->addOperator(std::make_shared<Precedence>(db, getGraphStorageFunc, join["segmentation-name"].asString(),
            minDist, maxDist),
            itLeft->second, itRight->second);
        }
        else
        {
          q->addOperator(std::make_shared<Precedence>(db, getGraphStorageFunc,
            minDist, maxDist),
            itLeft->second, itRight->second);
        }
      }
      else if (op == "Inclusion")
      {
        q->addOperator(std::make_shared<Inclusion>(db, getGraphStorageFunc), itLeft->second, itRight->second);
      }
      else if (op == "Overlap")
      {
        q->addOperator(std::make_shared<Overlap>(db, getGraphStorageFunc), itLeft->second, itRight->second);
      }
      else if (op == "IdenticalCoverage")
      {
        q->addOperator(std::make_shared<IdenticalCoverage>(db, getGraphStorageFunc), itLeft->second, itRight->second);
      }
      else if (op == "Dominance")
      {

        std::string name = join["name"].isString() ? join["name"].asString() : "";

        if (join["edgeAnnotations"].isArray() && join["edgeAnnotations"].size() > 0)
        {
          auto anno = getEdgeAnno(db, join["edgeAnnotations"][0]);
          q->addOperator(std::make_shared<Dominance>(name, getAllGraphStorageFunc, db.strings, anno),
            itLeft->second, itRight->second);

        }
        else
        {

          auto minDist = join["minDistance"].asUInt();
          auto maxDist = join["maxDistance"].asUInt();
          if (minDist == 0 && maxDist == 0)
          {
            // unlimited range
            minDist = 1;
            maxDist = uintmax;
          }

          q->addOperator(std::make_shared<Dominance>(name, getAllGraphStorageFunc, db.strings,
            minDist, maxDist),
            itLeft->second, itRight->second);
        }
      }
      else if (op == "Pointing")
      {

        std::string name = join["name"].asString();

        if (join["edgeAnnotations"].isArray() && join["edgeAnnotations"].size() > 0)
        {
          auto anno = getEdgeAnno(db, join["edgeAnnotations"][0]);
          q->addOperator(std::make_shared<Pointing>(name, getAllGraphStorageFunc, db.strings, anno),
            itLeft->second, itRight->second);

        }
        else
        {

          auto minDist = join["minDistance"].asUInt();
          auto maxDist = join["maxDistance"].asUInt();

          if (minDist == 0 && maxDist == 0)
          {
            // unlimited range
            minDist = 1;
            maxDist = uintmax;
          }

          q->addOperator(std::make_shared<Pointing>(name, getAllGraphStorageFunc, db.strings, minDist, maxDist),
            itLeft->second, itRight->second);
        }
      }

    }

  }
}

Annotation JSONQueryParser::getEdgeAnno(const DB& db, const Json::Value& edgeAnno)
{

  std::uint32_t ns = 0;
  std::uint32_t name = 0;
  std::uint32_t value = 0;

  if (edgeAnno["textMatching"].asString() == "EXACT_EQUAL")
  {
    if (edgeAnno["namespace"].isString())
    {
      std::string nsStr = edgeAnno["namespace"].asString();
      auto search = db.strings.findID(nsStr);
      // if string is not found set to an invalid value
      ns = search.first ? search.second : std::numeric_limits<std::uint32_t>::max();
    }
    if (edgeAnno["name"].isString())
    {
      std::string nameStr = edgeAnno["name"].asString();
      auto search = db.strings.findID(nameStr);
      // if string is not found set to an invalid value
      name = search.first ? search.second : std::numeric_limits<std::uint32_t>::max();
    }
    if (edgeAnno["value"].isString())
    {
      std::string valueStr = edgeAnno["value"].asString();
      auto search = db.strings.findID(valueStr);
      // if string is not found set to an invalid value
      value = search.first ? search.second : std::numeric_limits<std::uint32_t>::max();
    }
  }
  // TODO: what about regex?

  return Init::initAnnotation(name, value, ns);
}

bool JSONQueryParser::canReplaceRegex(const std::string& str) 
{
  // Characters that have a meaning according to
  // https://github.com/google/re2/wiki/Syntax
  // Characters used in not supported functions are not included.
  if(str.find_first_of(".[]\\|*+?{}()^$") == std::string::npos)
  {
    // No meta character found in string, might be replaced    
    RE2 regex(str);
    if(regex.ok())
    {
      return true;
    }
    else
    {
      // If there is an error during parsing this is still a regex (an invalid one).
      // Treating it like a exact string would not give the same result.
      return false;
    }
  }
  else
  {
    // contains special regex characters
    return false;
  }
}


JSONQueryParser::~JSONQueryParser()
{
}

