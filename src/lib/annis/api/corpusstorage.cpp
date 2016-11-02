#include "corpusstorage.h"

#include <boost/thread/lock_guard.hpp>
#include <boost/thread/shared_lock_guard.hpp>

using namespace annis;
using namespace annis::api;

CorpusStorage::CorpusStorage(std::string databaseDir)
  : databaseDir(databaseDir)
{
  cache = std::unique_ptr<DBCache>(new DBCache());
}

CorpusStorage::~CorpusStorage() {}

long long CorpusStorage::count(std::vector<std::string> corpora, std::string queryAsJSON)
{
  long long result = 0;

  // sort corpora by their name
  std::sort(corpora.begin(), corpora.end());

  for(const std::string& c : corpora)
  {
    std::shared_ptr<DB> db = cache->get(databaseDir + "/" + c, true);

    if(db)
    {
      boost::shared_lock_guard<DB> lock(*db);

      std::stringstream ss;
      ss << queryAsJSON;
      std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(*db, db->edges, ss);
      while(q->next())
      {
        result++;
      }
    }
  }
  return result;
}

CorpusStorage::CountResult CorpusStorage::countExtra(std::vector<std::string> corpora, std::string queryAsJSON)
{
  CountResult result = {0,0};

  std::set<std::uint32_t> documents;

  // sort corpora by their name
  std::sort(corpora.begin(), corpora.end());

  for(const std::string& c : corpora)
  {
    std::shared_ptr<DB> db = cache->get(databaseDir + "/" + c, true);

    if(db)
    {
      boost::shared_lock_guard<DB> lock(*db);

      std::stringstream ss;
      ss << queryAsJSON;
      std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(*db, db->edges, ss);
      while(q->next())
      {
        result.matchCount++;
        const std::vector<Match>& m = q->getCurrent();
        if(!m.empty())
        {
          const Match& n  = m[0];
          std::pair<bool, Annotation> anno = db->nodeAnnos.getNodeAnnotation(n.node, annis_ns, "document");
          if(anno.first)
          {
            documents.insert(anno.second.val);
          }
        }
      }
    }
  }

  result.documentCount = documents.size();
  return result;
}

std::vector<std::string> CorpusStorage::find(std::vector<std::string> corpora, std::string queryAsJSON, long long offset, long long limit)
{
  std::vector<std::string> result;

  long long counter = 0;

  // sort corpora by their name
  std::sort(corpora.begin(), corpora.end());

  for(const std::string& c : corpora)
  {
    std::shared_ptr<DB> db = cache->get(databaseDir + "/" + c, false);

    if(db)
    {
      boost::shared_lock_guard<DB> lock(*db);

      std::stringstream ss;
      ss << queryAsJSON;
      std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(*db, db->edges, ss);
      while(counter < (offset + limit) && q->next())
      {
        if(counter >= offset)
        {
          const std::vector<Match>& m = q->getCurrent();
          std::stringstream matchDesc;
          for(size_t i = 0; i < m.size(); i++)
          {
            const Match& n = m[i];

            if(n.anno.ns != 0 && n.anno.name != 0
               && n.anno.ns != db->getNamespaceStringID() && n.anno.name != db->getNodeNameStringID())
            {
              matchDesc << db->strings.str(n.anno.ns)
                << "::" << db->strings.str(n.anno.name)
                << "::";
            }

            matchDesc << "salt:/" << c << "/";
            matchDesc << db->getNodeDocument(n.node) << "/#" << db->getNodeName(n.node);

            if(i < m.size()-1)
            {
             matchDesc << " ";
            }
          }
          result.push_back(matchDesc.str());
        } // end if result in offset-limit range
        counter++;
      }
    }
  }

  return result;
}
