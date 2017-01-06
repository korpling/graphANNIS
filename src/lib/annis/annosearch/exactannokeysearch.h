#pragma once

#include <annis/annosearch/annotationsearch.h>

#include <annis/annostorage.h>

namespace annis
{

class ExactAnnoKeySearch : public AnnotationKeySearch
{
  using ItAnnoNode = AnnoStorage<nodeid_t>::InverseAnnoMap_t::const_iterator;
  using ItAnnoKey = btree::btree_map<AnnotationKey, std::uint64_t>::const_iterator;

public:
  /**
   * @brief Find all annotations.
   * @param db
   */
  ExactAnnoKeySearch(const DB& db);
  /**
   * @brief Find annotations by name
   * @param db
   * @param annoName
   */
  ExactAnnoKeySearch(const DB& db, const std::string& annoName);
  ExactAnnoKeySearch(const DB& db, const std::string& annoNamspace, const std::string& annoName);

  virtual ~ExactAnnoKeySearch();

  virtual bool next(Match& result) override;
  virtual void reset() override;

  const std::set<AnnotationKey>& getValidAnnotationKeys() override
  {
    if(!validAnnotationKeysInitialized)
    {
      initializeValidAnnotationKeys();
    }
    return validAnnotationKeys;
  }
  
  virtual std::int64_t guessMaxCount() const override;

  virtual std::string debugString() const override {return debugDescription;}

private:
  const DB& db;

  ItAnnoNode it;
  ItAnnoNode itBegin;
  ItAnnoNode itEnd;

  ItAnnoKey itKeyBegin;
  ItAnnoKey itKeyEnd;

  bool validAnnotationKeysInitialized;
  std::set<AnnotationKey> validAnnotationKeys;

  const std::string debugDescription;
private:
  void initializeValidAnnotationKeys();

};


} // end namespace annis
