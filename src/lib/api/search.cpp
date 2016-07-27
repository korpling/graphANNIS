#include <annis/api/search.h>

using namespace annis;
using namespace annis::api;

Search::Search(std::string databaseDir)
  : databaseDir(databaseDir)
{
  cache = std::unique_ptr<DBCache>(new DBCache());
}

Search::~Search() {}

long long Search::count(std::vector<std::string> corpora, std::string queryAsJSON)
{
  long long result = 0;

  // sort corpora by their name
  std::sort(corpora.begin(), corpora.end());

  for(const std::string& c : corpora)
  {
    std::weak_ptr<DB> dbWeakPtr = cache->get(databaseDir + "/" + c);

    if(std::shared_ptr<DB> db = dbWeakPtr.lock())
    {
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

std::vector<std::string> Search::find(std::vector<std::string> corpora, std::string queryAsJSON, long long offset, long long limit)
{
  std::vector<std::string> result;

  long long counter = 0;

  // sort corpora by their name
  std::sort(corpora.begin(), corpora.end());

  for(const std::string& c : corpora)
  {
    std::weak_ptr<DB> dbWeakPtr = cache->get(databaseDir + "/" + c);

    if(std::shared_ptr<DB> db = dbWeakPtr.lock())
    {
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

            matchDesc << "salt://" << c << "/";
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
