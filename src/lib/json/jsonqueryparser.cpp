/* 
 * File:   JSONQueryParser.cpp
 * Author: Thomas Krause <thomaskrause@posteo.de>
 * 
 * Created on 3. Januar 2016, 16:05
 */

#include <annis/json/jsonqueryparser.h>
#include <annis/annosearch/exactannovaluesearch.h>
#include <annis/annosearch/exactannokeysearch.h>
#include <annis/annosearch/regexannosearch.h>
#include <annis/operators/precedence.h>
#include <annis/operators/dominance.h>
#include <annis/operators/pointing.h>
#include <annis/operators/inclusion.h>
#include <annis/operators/overlap.h>
#include <annis/operators/identicalcoverage.h>
#include <map>

using namespace annis;

JSONQueryParser::JSONQueryParser()
{
}

std::shared_ptr<Query> JSONQueryParser::parse(const DB& db, std::istream& jsonStream, bool optimize)
{
  std::shared_ptr<Query> q = std::make_shared<Query>(db, optimize);

  // parse root as value
  Json::Value root;
  jsonStream >> root;

  // get the first alternative (we don't support more than one currently)
  const auto& alternatives = root["alternatives"];
  if (alternatives.size() != 0)
  {
    const auto& firstAlt = alternatives[0];

    // add all nodes
    const auto& nodes = firstAlt["nodes"];

    std::map<std::uint64_t, size_t> nodeIdToPos;
    for (auto it = nodes.begin(); it != nodes.end(); it++)
    {
      auto& n = *it;
      nodeIdToPos[std::stoull(it.name())] = parseNode(db, n, q);
    }

    // add all joins
    const auto& joins = firstAlt["joins"];
    for (auto it = joins.begin(); it != joins.end(); it++)
    {
      parseJoin(db, *it, q, nodeIdToPos);
    }


  }
  return q;
}

size_t JSONQueryParser::parseNode(const DB& db, const Json::Value node, std::shared_ptr<Query> q)
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
      return addNodeAnnotation(db, q, optStr(annis_ns), optStr(annis_tok),
        optStr(node["spannedText"]),
        optStr(node["spanTextMatching"]), true);
    }// end if token has spanned text
    else
    {
      // just search for any node
      return addNodeAnnotation(db, q, optStr(annis_ns), optStr(annis_node_name),
        optStr(), optStr());
    }
  } // end if special case

}

size_t JSONQueryParser::addNodeAnnotation(const DB& db,
  std::shared_ptr<Query> q,
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
}

void JSONQueryParser::parseJoin(const DB& db, const Json::Value join, std::shared_ptr<Query> q,
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
        q->addOperator(std::make_shared<Precedence>(db,
          minDist, maxDist),
          itLeft->second, itRight->second);
      }
      else if (op == "Inclusion")
      {
        q->addOperator(std::make_shared<Inclusion>(db), itLeft->second, itRight->second);
      }
      else if (op == "Overlap")
      {
        q->addOperator(std::make_shared<Overlap>(db), itLeft->second, itRight->second);
      }
      else if (op == "IdenticalCoverage")
      {
        q->addOperator(std::make_shared<IdenticalCoverage>(db), itLeft->second, itRight->second);
      }
      else if (op == "Dominance")
      {

        std::string name = join["name"].isString() ? join["name"].asString() : "";

        if (join["edgeAnnotations"].isArray() && join["edgeAnnotations"].size() > 0)
        {
          auto anno = getEdgeAnno(db, join["edgeAnnotations"][0]);
          q->addOperator(std::make_shared<Dominance>(db, "", name, anno),
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

          q->addOperator(std::make_shared<Dominance>(db,
            "", name,
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
          q->addOperator(std::make_shared<Pointing>(db, "", name, anno),
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

          q->addOperator(std::make_shared<Pointing>(db,
            "", name, minDist, maxDist),
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
      if (search.first)
      {
        ns = search.second;
      }
    }
    if (edgeAnno["name"].isString())
    {
      std::string nameStr = edgeAnno["name"].asString();
      auto search = db.strings.findID(nameStr);
      if (search.first)
      {
        name = search.second;
      }
    }
    if (edgeAnno["value"].isString())
    {
      std::string valueStr = edgeAnno["value"].asString();
      auto search = db.strings.findID(valueStr);
      if (search.first)
      {
        value = search.second;
      }
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

