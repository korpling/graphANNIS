#ifndef STRINGSTORAGE_H
#define STRINGSTORAGE_H

#include <string>
#include <stx/btree_map>

namespace annis
{

class StringStorage
{
public:
  StringStorage();

  const std::string& str(std::uint32_t id) const
  {
    typedef stx::btree_map<std::uint32_t, std::string>::const_iterator ItType;
    ItType it = stringStorageByID.find(id);
    if(it != stringStorageByID.end())
    {
      return it->second;
    }
    else
    {
      throw("Unknown string ID");
    }
  }

  std::pair<bool, std::uint32_t> findID(const std::string& str) const
  {
    typedef stx::btree_map<std::string, std::uint32_t>::const_iterator ItType;
    std::pair<bool, std::uint32_t> result;
    result.first = false;
    ItType it = stringStorageByValue.find(str);
    if(it != stringStorageByValue.end())
    {
      result.first = true;
      result.second = it->second;
    }
    return result;
  }

  std::uint32_t add(const std::string& str);

  void clear();
  bool load(const std::string& dirPath);
  bool save(const std::string &dirPath);
  size_t size() {return stringStorageByID.size();}

private:
  stx::btree_map<std::uint32_t, std::string> stringStorageByID;
  stx::btree_map<std::string, std::uint32_t> stringStorageByValue;

};
}

#endif // STRINGSTORAGE_H
