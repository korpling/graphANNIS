/* 
 * File:   nodeannostorage.h
 * Author: thomas
 *
 * Created on 14. Januar 2016, 13:53
 */

#pragma once

#include <stx/btree_map>
#include <stx/btree_multimap>
#include <set>
#include <list>
#include <boost/optional.hpp>

#include <annis/types.h>
#include <annis/stringstorage.h>

namespace annis {
  class NodeAnnoStorage
  {
    friend class DB;
    friend class ExactAnnoValueSearch;
    friend class ExactAnnoKeySearch;
    friend class RegexAnnoSearch;


  public:
    NodeAnnoStorage(StringStorage& strings);
    NodeAnnoStorage(const NodeAnnoStorage& orig) = delete;

    void addNodeAnnotation(nodeid_t nodeID, Annotation& anno)
    {
      nodeAnnotations.insert2({nodeID, anno.name, anno.ns}, anno.val);
      inverseNodeAnnotations.insert2(anno, nodeID);
      nodeAnnoKeys.insert({anno.name, anno.ns});
    }

    inline std::list<Annotation> getNodeAnnotationsByID(const nodeid_t &id) const
    {
      typedef stx::btree_map<NodeAnnotationKey, uint32_t>::const_iterator AnnoIt;

      NodeAnnotationKey lowerAnno = {id, 0, 0};
      NodeAnnotationKey upperAnno = {id, uintmax, uintmax};

      std::list<Annotation> result;
      std::pair<AnnoIt, AnnoIt> itRange = {
        nodeAnnotations.lower_bound(lowerAnno),
        nodeAnnotations.upper_bound(upperAnno)
      };

      for (AnnoIt it = itRange.first;
        it != itRange.second; it++)
      {
        const auto& key = it.key();
        result.push_back({key.anno_name, key.anno_ns, it.data()});
      }

      return result;
    }

    inline std::pair<bool, Annotation> getNodeAnnotation(const nodeid_t &id, const std::uint32_t& nsID, const std::uint32_t& nameID) const
    {
      auto it = nodeAnnotations.find({id, nameID, nsID});

      if (it != nodeAnnotations.end())
      {
        return
        {
          true,
          {
            nameID, nsID, it.data()
          }
        };
      }
      return
      {
        false,
        {
          0, 0, 0
        }
      };
    }

    inline std::pair<bool, Annotation> getNodeAnnotation(const nodeid_t &id, const std::string& ns, const std::string& name) const
    {
      std::pair<bool, std::uint32_t> nsID = strings.findID(ns);
      std::pair<bool, std::uint32_t> nameID = strings.findID(name);

      if (nsID.first && nameID.first)
      {
        return getNodeAnnotation(id, nsID.second, nameID.second);
      }

      std::pair<bool, Annotation> noResult;
      noResult.first = false;
      return noResult;
    }
    
    void calculateStatistics();
    size_t guessCount(const std::string& ns, const std::string& name, const std::string& val);
    size_t guessCount(const std::string& name, const std::string& val);

    bool load(std::string dirPath);
    bool save(std::string dirPath);
    void clear();

    virtual ~NodeAnnoStorage();
  private:
    /**
     * @brief Maps a fully qualified annotation name for a node to an annotation value
     */
    stx::btree_map<NodeAnnotationKey, uint32_t> nodeAnnotations;
    stx::btree_multimap<Annotation, nodeid_t> inverseNodeAnnotations;
    std::set<AnnotationKey> nodeAnnoKeys;

    StringStorage& strings;
    
    /* statistical information */
    std::map<AnnotationKey, std::vector<std::string>> histogramBounds;
  private:
    /**
     * Internal function for getting an estimation about the number of matches for a certain range of annotation value
     * @param nsID The namespace part of the annotation key. Can be empty (in this case all annotations with the correct name are used).
     * @param nameID The name part of the annotation key.
     * @param lowerVal Inclusive starting point for the value range.
     * @param upperVal Exclusive or inclusive end point for the value range.
     * @param if true upperVal is inclusive, otherwise it is exclusive
     * @return 
     */
    size_t guessCount(boost::optional<std::uint32_t> nsID, std::uint32_t nameID, const std::string& lowerVal,
      const std::string& upperVal, bool upperInclusive = true);
  };
}


