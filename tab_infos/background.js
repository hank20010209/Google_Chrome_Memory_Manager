async function sendTabInfoToServer(tabs) {
  let tabsJson = JSON.stringify(tabs, null, 4);

  try {
    const response = await fetch('http://127.0.0.1:8080', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: tabsJson,
    });

    if (response.ok) {
      console.log('Tab info sent successfully');
    } else {
      console.error('Failed to send tab info:', response.statusText);
    }
  } catch (error) {
    console.error('Error sending tab info:', error);
  }
}

async function saveTabInfos(tabId, changeInfo, tab) {
  if (changeInfo.status === "complete" && tab.title) {
    let tabs = await chrome.tabs.query({});

    await Promise.all(tabs.map(async tab => {
      tab.pid = await chrome.processes.getProcessIdForTab(tab.id);
      return tab;
    }));

    sendTabInfoToServer(tabs);
  }
}

async function onActivatedHandler() {
    let tabs = await chrome.tabs.query({});
    await Promise.all(tabs.map(async tab => {
      tab.pid = await chrome.processes.getProcessIdForTab(tab.id);
      return tab;
    }));
    sendTabInfoToServer(tabs);
}

async function handleTabRemoved(tabId) {
  let tabs = await chrome.tabs.query({});

  await Promise.all(tabs.map(async tab => {
    tab.pid = await chrome.processes.getProcessIdForTab(tab.id);
    return tab;
  }));

  sendTabInfoToServer(tabs);
}

chrome.tabs.onUpdated.addListener(saveTabInfos);
chrome.tabs.onActivated.addListener(onActivatedHandler);
chrome.tabs.onRemoved.addListener(handleTabRemoved);
setInterval(() => {
  onActivatedHandler().catch(error => console.error('Error in periodic onActivatedHandler:', error));
}, 3000);