#include "corpusstoragemanager.h"

#include <boost/thread/lock_guard.hpp>
#include <boost/thread/shared_lock_guard.hpp>
#include <boost/filesystem.hpp>
#include <boost/algorithm/string/predicate.hpp>

#include <fstream>
#include <thread>
#include <cereal/archives/binary.hpp>

#include <annis/db.h>

using namespace annis;
using namespace annis::api;

namespace bf = boost::filesystem;

CorpusStorageManager::CorpusStorageManager(std::string databaseDir, size_t maxAllowedCacheSize)
  : databaseDir(databaseDir), maxAllowedCacheSize(maxAllowedCacheSize)
{
}

CorpusStorageManager::~CorpusStorageManager() {}

long long CorpusStorageManager::count(std::vector<std::string> corpora, std::string queryAsJSON)
{
  long long result = 0;

  // sort corpora by their name
  std::sort(corpora.begin(), corpora.end());

  for(const std::string& c : corpora)
  {
    std::shared_ptr<DBLoader> loader = getCorpusFromCache(c);

    if(loader)
    {
      boost::shared_lock_guard<DBLoader> lock(*loader);

      DB& db = loader->get();

      std::stringstream ss;
      ss << queryAsJSON;
      std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(db, db.edges, ss);
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
    std::shared_ptr<DBLoader> loader = getCorpusFromCache(c);

    if(loader)
    {
      boost::shared_lock_guard<DBLoader> lock(*loader);

      std::stringstream ss;
      ss << queryAsJSON;
      std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(loader->get(), loader->get().edges, ss);
      while(q->next())
      {
        result.matchCount++;
        const std::vector<Match>& m = q->getCurrent();
        if(!m.empty())
        {
          const Match& n  = m[0];
          std::pair<bool, Annotation> anno = loader->get().nodeAnnos.getNodeAnnotation(n.node, annis_ns, "document");
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
    std::shared_ptr<DBLoader> loader = getCorpusFromCache(c);

    if(loader)
    {
      boost::shared_lock_guard<DBLoader> lock(*loader);

      std::stringstream ss;
      ss << queryAsJSON;
      std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(loader->get(), loader->get().edges, ss);
      while((limit <= 0 || counter < (offset + limit)) && q->next())
      {
        if(counter >= offset)
        {
          const std::vector<Match>& m = q->getCurrent();
          std::stringstream matchDesc;
          for(size_t i = 0; i < m.size(); i++)
          {
            const Match& n = m[i];

            DB& db = loader->get();

            if(n.anno.ns != 0 && n.anno.name != 0
               && n.anno.ns != db.getNamespaceStringID() && n.anno.name != db.getNodeNameStringID())
            {
              matchDesc << db.strings.str(n.anno.ns)
                << "::" << db.strings.str(n.anno.name)
                << "::";
            }

            matchDesc << "salt:/" << c << "/";
            matchDesc << db.getNodeDocument(n.node) << "/#" << db.getNodeName(n.node);

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

   killBackgroundWriter(corpus);

   if(!update.isConsistent())
   {
      // Always mark the update state as consistent, even if caller forgot this.
      update.finish();
   }

   bf::path corpusPath = bf::path(databaseDir) / corpus;

   // we have to make sure that the corpus is fully loaded (with all components) before we can apply the update.
   std::shared_ptr<DBLoader> loader = getCorpusFromCache(corpus);

   if(loader)
   {
      boost::lock_guard<DBLoader> lock(*loader);

      DB& db = loader->getFullyLoaded();
      try {

         db.update(update);

         // if successfull write log
         bf::create_directories(corpusPath / "current");
         std::ofstream logStream((corpusPath / "current" / "update_log.cereal").string());
         cereal::BinaryOutputArchive ar(logStream);
         ar(update);

         // Until now only the write log is persisted. Start a background thread that writes the whole
         // corpus to the folder (without the need to apply the write log).
         startBackgroundWriter(corpus, loader);

      } catch (...)
      {
         db.load(databaseDir + "/" + corpus);
      }
   }
}

std::vector<std::string> CorpusStorageManager::list()
{
  std::vector<std::string> result;

  bf::path root(databaseDir);

  if(bf::is_directory(root))
  {
    for(bf::directory_iterator it(root); it != bf::directory_iterator(); ++it)
    {
      if(bf::is_directory(it->status()))
      {
        bf::path corpusPath = it->path();
        result.push_back(corpusPath.filename().string());
      }
    }
  }
  return result;
}

void CorpusStorageManager::importCorpus(std::string pathToCorpus, std::string newCorpusName)
{

   // load an existing corpus or create a our common database directory
   std::shared_ptr<DBLoader> loader = getCorpusFromCache(newCorpusName);
   if(loader)
   {
      boost::lock_guard<DBLoader> lock(*loader);
      DB& db = loader->get();
      // load the corpus data from the external location
      db.load(pathToCorpus);
      // make sure the corpus is properly saved at least once (so it is in a consistent state)
      db.save((bf::path(databaseDir) / newCorpusName).string());
   }
}

void CorpusStorageManager::exportCorpus(std::string corpusName, std::string exportPath)
{
  std::shared_ptr<DBLoader> loader = getCorpusFromCache(corpusName);
  if(loader)
  {
     boost::shared_lock_guard<DBLoader> lock(*loader);
     // load the corpus data from the external location
     loader->getFullyLoaded().save(exportPath);
  }
}

bool CorpusStorageManager::deleteCorpus(std::string corpusName)
{
  bf::path root(databaseDir);
  bf::path corpusPath  = root / corpusName;

  // This will block until the internal map is available, thus do this before locking the database to avoid any deadlock
  killBackgroundWriter(corpusPath.string());

  // Get the DB and hold a lock on it until we are finished.
  // Preloading all components so we are able to restore the complete DB if anything goes wrong.
  std::shared_ptr<DBLoader> loader = getCorpusFromCache(corpusPath.string());
  if(loader)
  {

    boost::lock_guard<DBLoader> lock(*loader);

    DB& db = loader->getFullyLoaded();

    try
    {
      // delete the corpus on the disk first, if we are interrupted the data is still in memory and can be restored
      bf::remove_all(corpusPath);
    }
    catch(...)
    {
      // if anything goes wrong write the corpus back to it's original location to have a consistent state
      db.save(corpusPath.string());

      return false;
    }


    // delete the corpus from the cache and thus from memory
    std::lock_guard<std::mutex> lockCorpusCache(mutex_corpusCache);
    corpusCache.erase(corpusName);

    return true;

  }
  return false;
}

void CorpusStorageManager::startBackgroundWriter(std::string corpus, std::shared_ptr<DBLoader>& loader)
{
  bf::path root = bf::path(databaseDir) / corpus;

  std::lock_guard<std::mutex> lock(mutex_writerThreads);
  writerThreads[corpus] = boost::thread([loader, root] () {

    // Get a read-lock for the database. The thread is started from another function which will have the database locked,
    // thus this thread will only really start as soon as the calling function has returned.
    // We start as a read-lock since it is safe to read the in-memory representation (and we won't change it)
    boost::shared_lock_guard<DBLoader> lock(*loader);

    // We could have been interrupted right after we waited for the lock, so check here just to be sure.
    boost::this_thread::interruption_point();


    DB& db = loader->getFullyLoaded();

    boost::this_thread::interruption_point();

    // Move the old corpus to the backup sub-folder. When the corpus is loaded again and there is backup folder
    // the backup will be used instead of the original possible corrupted files.
    // The current version is only the real one if no backup folder exists. If there is a backup folder
    // there is nothing to do since the backup already contains the last consistent version.
    // A sub-folder is used to ensure that all directories are on the same file system and moving (instead of copying)
    // is possible.
    if(!bf::exists(root / "backup"))
    {
      bf::rename(root / "current", root / "backup");
    }

    boost::this_thread::interruption_point();

    // Save the complete corpus without the write log to the target location
    db.save(root.string());

    boost::this_thread::interruption_point();

    // remove the backup folder (since the new folder was completly written)
    bf::remove_all(root / "backup");

  });

}

void CorpusStorageManager::killBackgroundWriter(std::string corpus)
{
  std::lock_guard<std::mutex> lock(mutex_writerThreads);
  auto itThread = writerThreads.find(corpus);
  if(itThread != writerThreads.end())
  {
    itThread->second.interrupt();

    // wait until thread is finished
    itThread->second.join();

    writerThreads.erase(itThread);
  }
}

std::shared_ptr<DBLoader> CorpusStorageManager::getCorpusFromCache(std::string name)
{
  std::lock_guard<std::mutex> lock(mutex_corpusCache);

  std::shared_ptr<DBLoader> result;

  auto it = corpusCache.find(name);

  if(it == corpusCache.end())
  {
    // create a new DB, load its content from disk and put it into cache
    result = std::make_shared<DBLoader>((bf::path(databaseDir) / name).string());
    corpusCache[name] =  result;
  }
  else
  {
    result = it->second;
  }

  return result;
}
