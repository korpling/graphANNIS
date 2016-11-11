#include "corpusstoragemanager.h"

#include <boost/thread/lock_guard.hpp>
#include <boost/thread/shared_lock_guard.hpp>
#include <boost/filesystem.hpp>

#include <fstream>
#include <thread>
#include <cereal/archives/binary.hpp>

#include <annis/db.h>

using namespace annis;
using namespace annis::api;

CorpusStorageManager::CorpusStorageManager(std::string databaseDir)
  : databaseDir(databaseDir)
{
  cache = std::unique_ptr<DBCache>(new DBCache());
}

CorpusStorageManager::~CorpusStorageManager() {}

long long CorpusStorageManager::count(std::vector<std::string> corpora, std::string queryAsJSON)
{
  long long result = 0;

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
      while(q->next())
      {
        result++;
      }
    }
  }
  return result;
}

CorpusStorageManager::CountResult CorpusStorageManager::countExtra(std::vector<std::string> corpora, std::string queryAsJSON)
{
  CountResult result = {0,0};

  std::set<std::uint32_t> documents;

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

std::vector<std::string> CorpusStorageManager::find(std::vector<std::string> corpora, std::string queryAsJSON, long long offset, long long limit)
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
      while((limit <= 0 || counter < (offset + limit)) && q->next())
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

void CorpusStorageManager::applyUpdate(std::string corpus, GraphUpdate &update)
{
   if(!update.isConsistent())
   {
      // Always mark the update state as consistent, even if caller forgot this.
      update.finish();
   }

   // we have to make sure that the corpus is fully loaded (with all components) before we can apply the update.
   std::shared_ptr<DB> db = cache->get(databaseDir + "/" + corpus, true);

   if(db)
   {
      boost::shared_lock_guard<DB> lock(*db);


      try {

         db->update(update);

         // if successfull write log
         boost::filesystem::path corpusDir(databaseDir);
         corpusDir = corpusDir / corpus;
         boost::filesystem::create_directories(corpusDir);
         std::ofstream logStream((corpusDir / "update_log.cereal").string());
         cereal::BinaryOutputArchive ar(logStream);
         ar(update);

      } catch (...)
      {
         db->load(databaseDir + "/" + corpus);
      }

   }
}

void CorpusStorageManager::loadExternalCorpus(std::string pathToCorpus, std::string newCorpusName)
{
   boost::filesystem::path internalPath = boost::filesystem::path(databaseDir) / newCorpusName;


   // load an existing corpus or create a our common database directory
   std::shared_ptr<DB> db = cache->get(internalPath.string(), true);
   if(db)
   {
      boost::shared_lock_guard<DB> lock(*db);
      // load the corpus data from the external location
      db->load(pathToCorpus);
      // make sure the corpus is properly saved at least once (so it is in a consistent state)
      db->save(internalPath.string());
   }
}
