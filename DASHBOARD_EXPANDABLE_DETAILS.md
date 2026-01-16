# Dashboard Expandable Details Feature

## Overview

The MeCP dashboard now includes expandable detail views for each API call and error, allowing you to view full request/response data with a single click.

## Features

### 🔍 Expandable Rows
- **Click anywhere on a row** to expand details
- **Click "View Details" button** for explicit expansion
- **Click again** to collapse

### 📋 What You Can See

#### Recent API Calls
Each expandable row shows:
- **Request Parameters**: Full JSON request with syntax highlighting
- **Response Data**: Complete JSON response with formatting
- **Additional Information**: ID, timestamp, duration, status, error messages, client info
- **Copy Buttons**: One-click copy for both request and response

#### Recent Errors
Each expandable error shows:
- **Request Parameters**: What was sent when the error occurred
- **Error Response**: Full error response with details
- **Error Details**: Complete error information
- **Copy Buttons**: Copy request or error response

### 🎨 Visual Design
- **Dark-themed JSON viewer** with syntax highlighting
- **Grid layout** for request/response side-by-side
- **Scrollable content** for large responses
- **Copy feedback** with green "Copied!" confirmation

## How to Use

### Viewing Details

**Option 1: Click anywhere on the row**
```
Click → Row expands → Shows full details
```

**Option 2: Click "View Details" button**
```
Click button → Row expands → Shows formatted data
```

**Option 3: Click again to collapse**
```
Click again → Row collapses → Returns to compact view
```

### Copying Data

1. **Expand a row** to see details
2. **Click "Copy" button** next to Request or Response
3. **Button turns green** with "Copied!" text
4. **Paste anywhere** - data is in your clipboard

### Navigation

- **Scroll within JSON viewer** if content is large
- **Auto-formatted JSON** for easy reading
- **Syntax-highlighted** code blocks

## UI Elements

### Buttons

**View Details**
- Purple button in Actions column
- Expands/collapses the detail row
- Hover effect for interactivity

**Copy**
- Green button next to each section title
- Copies content to clipboard
- Shows "Copied!" confirmation for 2 seconds

### Detail Sections

**Request Parameters**
```json
{
  "protocolVersion": "2024-11-05",
  "clientInfo": {
    "name": "test-client",
    "version": "1.0.0"
  }
}
```

**Response Data**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {...},
    "serverInfo": {...}
  }
}
```

**Additional Information**
- ID: Log entry ID
- Timestamp: When the call was made
- Duration: Response time in milliseconds
- Status: Success or error
- Error: Error message (if applicable)
- Client: Client information (if provided)

## Examples

### Example 1: View Successful API Call

1. Open dashboard: `http://127.0.0.1:3000/dashboard`
2. Find a successful call in "Recent API Calls"
3. Click the row or "View Details" button
4. See:
   - **Request**: What parameters were sent
   - **Response**: What the server returned
   - **Info**: Timing and metadata

### Example 2: Debug an Error

1. Go to "Recent Errors" section
2. Click on an error row
3. See:
   - **Request**: What was sent that caused the error
   - **Error Response**: Full error details including code and message
   - **Error Details**: Additional context

### Example 3: Copy Request for Testing

1. Expand any API call
2. Click "Copy" next to "Request Parameters"
3. Button shows "Copied!"
4. Paste into your test client or curl command

### Example 4: Copy Response for Analysis

1. Expand an API call
2. Click "Copy" next to "Response Data"
3. Paste into JSON validator or analysis tool
4. Use for debugging or documentation

## Keyboard Shortcuts

- **Click row**: Expand/collapse
- **Escape**: (Browser default) No special handling yet
- **Ctrl+C**: After selecting text in JSON viewer

## Responsive Design

### Desktop
- **Two-column layout**: Request and response side-by-side
- **Wider JSON viewers**: More readable code
- **Full information grid**: All metadata visible

### Mobile/Tablet
- **Single-column layout**: Request and response stacked
- **Touch-friendly buttons**: Larger touch targets
- **Scrollable JSON**: Horizontal scroll for long lines

## Performance

### Efficient Rendering
- **Lazy expansion**: Details only rendered when expanded
- **No re-fetch**: Uses data already loaded
- **Smooth animations**: CSS transitions
- **Fast copy**: Native clipboard API

### Memory Usage
- **Only visible data**: Collapsed rows have minimal DOM
- **Limited to 100 logs**: Dashboard pagination
- **Garbage collected**: Closed rows cleanup automatically

## Styling Details

### JSON Viewer
- **Background**: Dark gray (#2d3748)
- **Text color**: Light gray (#e2e8f0)
- **Font**: Courier New monospace
- **Padding**: 15px
- **Max height**: 400px (scrollable)
- **Border radius**: 6px

### Buttons
- **Expand button**: Purple (#667eea)
- **Copy button**: Green (#48bb78)
- **Hover states**: Darker shades
- **Active state**: Slight scale down (0.95)

### Detail Row
- **Background**: Light gray (#f8f9fa)
- **Border**: Top border for separation
- **Padding**: 20px
- **Transition**: Smooth display toggle

## Tips & Tricks

### 1. Quick Comparison
- Open multiple rows
- Compare request/response patterns
- Identify differences quickly

### 2. Copy for Curl
```bash
# Copy request from dashboard
# Then use in curl:
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '<paste-request-here>'
```

### 3. Debug Flow
1. Find error in Recent Errors
2. Expand to see what was sent
3. Compare with successful calls
4. Identify the difference

### 4. Documentation
- Copy successful responses
- Use as examples in documentation
- Show expected vs actual results

### 5. Testing Validation
- Expand recent calls
- Verify response format
- Check all required fields present

## Browser Compatibility

### Tested On
- ✅ Chrome/Edge (latest)
- ✅ Firefox (latest)
- ✅ Safari (latest)
- ✅ Mobile browsers

### Required Features
- **Clipboard API**: For copy functionality
- **CSS Grid**: For responsive layout
- **CSS Transitions**: For smooth animations
- **ES6 JavaScript**: For modern syntax

## Limitations

### Current Limitations
- **No search within JSON**: Use browser search (Ctrl+F)
- **No syntax highlighting colors**: Plain monospace
- **No JSON tree view**: Flat text only
- **Manual refresh**: Not real-time (5s auto-refresh)

### Planned Improvements
- [ ] JSON syntax highlighting with colors
- [ ] Collapsible JSON tree view
- [ ] Search/filter within details
- [ ] Direct edit and resend capability
- [ ] Export to file functionality

## Troubleshooting

### Rows Won't Expand

**Check JavaScript console**: Press F12 → Console
```javascript
// Look for errors
// Common: "toggleDetails is not defined"
```

**Hard refresh**: Ctrl+Shift+R
```bash
# Clears cache and reloads
```

### Copy Doesn't Work

**Check HTTPS**: Clipboard API requires secure context
```
http://localhost:3000 ✅ Works
http://127.0.0.1:3000 ✅ Works  
http://your-ip:3000 ❌ Might not work
https://your-domain:3000 ✅ Works
```

**Browser permissions**: Check clipboard permissions

### JSON Not Formatted

**Check data**: Verify response_data exists
```sql
SELECT id, response_data 
FROM history_logs 
WHERE response_data IS NOT NULL 
LIMIT 1;
```

**Invalid JSON**: formatJSON handles gracefully
```javascript
// Shows raw text if JSON.parse fails
```

### Details Not Showing

**Check migration**: Ensure response_data column exists
```bash
./scripts/migrate-database.sh
```

**Restart server**: After migration
```bash
cargo run --release
```

## Accessibility

### Screen Reader Support
- **Button labels**: Clear "View Details" text
- **Table structure**: Proper HTML table markup
- **Semantic HTML**: Header tags for sections

### Keyboard Navigation
- **Tab navigation**: Through buttons and rows
- **Enter/Space**: Activate buttons
- **Click events**: Keyboard accessible

### Visual Clarity
- **High contrast**: Dark text on light background
- **Readable fonts**: Sans-serif for UI, monospace for code
- **Clear hierarchy**: Headers and sections

## Security Considerations

### Sensitive Data
- **Be aware**: Full request/response visible
- **Access control**: Secure dashboard in production
- **Data retention**: Clean up old logs

### Clipboard Security
- **User initiated**: Copy only on button click
- **No auto-copy**: No background clipboard access
- **Clear feedback**: User knows what was copied

## Integration Examples

### Python Script to Analyze
```python
import requests

# Get logs with details
response = requests.get('http://127.0.0.1:3000/api/logs')
logs = response.json()['logs']

# Analyze request/response pairs
for log in logs:
    if log['request_params'] and log['response_data']:
        print(f"Method: {log['method']}")
        print(f"Request: {log['request_params']}")
        print(f"Response: {log['response_data']}")
        print("---")
```

### Curl to Get Specific Log
```bash
# Get logs
curl -s http://127.0.0.1:3000/api/logs | jq '.logs[0] | {
  method,
  request: .request_params,
  response: .response_data
}'
```

## Summary

The expandable details feature provides:

✅ **Full visibility** - See complete request/response data  
✅ **Easy access** - One-click expansion  
✅ **Copy functionality** - Quick data extraction  
✅ **Professional UI** - Clean, modern design  
✅ **Responsive** - Works on all devices  
✅ **Fast** - No additional API calls  
✅ **Intuitive** - Natural interaction pattern  
✅ **Debug-friendly** - Perfect for troubleshooting  

**Start using it now!** Just click on any row in the Recent API Calls or Recent Errors tables.

---

**Feature Version**: 1.0  
**Release Date**: 2026-01-16  
**Status**: Production Ready ✅
