Here are examples of how to call your MCP server from JavaScript using both POST and GET requests:

## POST Request Examples

The MCP protocol uses POST requests with JSON-RPC 2.0 format:

```javascript
// 1. Initialize the MCP connection
async function initializeMCP() {
  const response = await fetch('http://127.0.0.1:8766/mcp', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method: 'initialize',
      params: {
        protocolVersion: '2024-11-05',
        capabilities: {
          roots: {
            listChanged: true
          }
        },
        clientInfo: {
          name: 'my-client',
          version: '1.0.0'
        }
      }
    })
  });
  
  const data = await response.json();
  console.log('Initialize response:', data);
  return data;
}

// 2. Search for text (semantic search)
async function searchText(query, limit = 10) {
  const response = await fetch('http://127.0.0.1:8766/mcp', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 2,
      method: 'tools/call',
      params: {
        name: 'search_text',
        arguments: {
          query: query,
          limit: limit,
          with_payload: true
        }
      }
    })
  });
  
  const data = await response.json();
  console.log('Search results:', data);
  return data;
}

// 3. Filter search (search with metadata filter)
async function filterSearch(query, filterField, filterValue) {
  const response = await fetch('http://127.0.0.1:8766/mcp', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 3,
      method: 'tools/call',
      params: {
        name: 'filter_search',
        arguments: {
          query: query,
          filter_field: filterField,
          filter_value: filterValue,
          limit: 10
        }
      }
    })
  });
  
  const data = await response.json();
  return data;
}

// 4. Keyword search (semantic + keyword filtering)
async function keywordSearch(query, keywords) {
  const response = await fetch('http://127.0.0.1:8766/mcp', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 4,
      method: 'tools/call',
      params: {
        name: 'keyword_search',
        arguments: {
          query: query,
          must_contain_keywords: keywords,
          limit: 10,
          with_payload: true
        }
      }
    })
  });
  
  const data = await response.json();
  return data;
}

// 5. Get collection info
async function getCollectionInfo() {
  const response = await fetch('http://127.0.0.1:8766/mcp', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 5,
      method: 'tools/call',
      params: {
        name: 'get_collection_info',
        arguments: {}
      }
    })
  });
  
  const data = await response.json();
  return data;
}

// 6. Count points
async function countPoints(exact = true) {
  const response = await fetch('http://127.0.0.1:8766/mcp', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 6,
      method: 'tools/call',
      params: {
        name: 'count_points',
        arguments: {
          exact: exact
        }
      }
    })
  });
  
  const data = await response.json();
  return data;
}

// 7. Scroll through points
async function scrollPoints(limit = 10, offset = null) {
  const response = await fetch('http://127.0.0.1:8766/mcp', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 7,
      method: 'tools/call',
      params: {
        name: 'scroll_points',
        arguments: {
          limit: limit,
          offset: offset,
          with_payload: true
        }
      }
    })
  });
  
  const data = await response.json();
  return data;
}

// 8. List available tools
async function listTools() {
  const response = await fetch('http://127.0.0.1:8766/mcp', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 8,
      method: 'tools/list',
      params: {}
    })
  });
  
  const data = await response.json();
  return data;
}

// 9. Read a resource
async function readResource(uri) {
  const response = await fetch('http://127.0.0.1:8766/mcp', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 9,
      method: 'resources/read',
      params: {
        uri: uri
      }
    })
  });
  
  const data = await response.json();
  return data;
}

// Usage example:
async function main() {
  try {
    // Initialize
    await initializeMCP();
    
    // Search for similar documents
    const searchResults = await searchText('machine learning algorithms', 5);
    console.log('Search:', searchResults);
    
    // Filter by username
    const filteredResults = await filterSearch(
      'data science',
      'username',
      'john_doe'
    );
    console.log('Filtered:', filteredResults);
    
    // Keyword search
    const keywordResults = await keywordSearch(
      'neural networks',
      'tensorflow pytorch'
    );
    console.log('Keywords:', keywordResults);
    
    // Get collection stats
    const info = await getCollectionInfo();
    console.log('Collection info:', info);
    
    // Count total points
    const count = await countPoints();
    console.log('Total points:', count);
    
    // Read collection resource
    const resource = await readResource('qdrant://collection');
    console.log('Resource:', resource);
    
  } catch (error) {
    console.error('Error:', error);
  }
}

main();
```

## Important Notes:

1. **JSON-RPC 2.0 Format**: All requests must follow the JSON-RPC 2.0 specification with `jsonrpc`, `id`, `method`, and `params` fields.

2. **No GET Requests**: MCP protocol uses POST requests exclusively. GET requests are not supported for the MCP endpoint.

3. **Error Handling**: Always check for errors in the response:
```javascript
const data = await response.json();
if (data.error) {
  console.error('MCP Error:', data.error);
  throw new Error(data.error.message);
}
```

4. **CORS**: If calling from a browser, you may need to configure CORS in your Rust server or use a proxy.

5. **Session Management**: The server uses `LocalSessionManager`, so each connection is stateless. You don't need to maintain session tokens.
