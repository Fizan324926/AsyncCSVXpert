import React, { useState, useRef } from 'react';
import { usePapaParse } from 'react-papaparse';
import './App.css';
import { CSVLink } from 'react-csv';
import { Pie } from 'react-chartjs-2';
import { Chart as ChartJS, ArcElement, Tooltip, Legend, CategoryScale, LinearScale } from 'chart.js';

ChartJS.register(ArcElement, Tooltip, Legend, CategoryScale, LinearScale);

function App() {
  const { readString } = usePapaParse();
  const [csvData, setCsvData] = useState(null);
  const [results, setResults] = useState([]);
  const [progress, setProgress] = useState(0);
  const [totalRecords, setTotalRecords] = useState(0);
  const [totalRecordsProcessed, setTotalRecordsProcessed] = useState(0);
  const [successCount, setSuccessCount] = useState(0);
  const [unsuccessCount, setUnsuccessCount] = useState(0);
  const [showProgress, setShowProgress] = useState(false);
  const [showCharts, setShowCharts] = useState(false);
  const [validationMessage, setValidationMessage] = useState('');
  const fileInputRef = useRef(null);
  const [fileName, setFileName] = useState('default_results.csv');
  const [processing, setProcessing] = useState(false);

  const handleFileDrop = (e) => {
    e.preventDefault();
    const file = e.dataTransfer.files[0];
    handleFileUpload(file);
  };

  const handleFileClick = () => {
    fileInputRef.current.click();
  };

  const handleFileChange = (e) => {
    const file = e.target.files[0];
    handleFileUpload(file);
  };

  const handleFileUpload = (file) => {
    if (!file || file.type !== 'text/csv') {
      alert('Please upload a valid CSV file.');
      return;
    }

    const reader = new FileReader();
    reader.onload = (event) => {
      const fileName = file.name;
      setFileName(`results_${fileName}`);
      readString(event.target.result, {
        header: true,
        complete: (result) => {
          const seen = new Set();
          let nullRecordsCount = 0;
          let duplicateRecordsCount = 0;

          const validData = result.data.filter((row) => {
            try {
              const keys = Object.keys(row);
              const key0 = row[keys[0]];
              const key1 = row[keys[1]];
              const isValid = keys.length >= 2 && key0 && key1;

              if (isValid) {
                const duplicateKey = `${key0}-${key1}`;
                if (seen.has(duplicateKey)) {
                  duplicateRecordsCount++;
                  return false;
                }
                seen.add(duplicateKey);
              } else {
                nullRecordsCount++;
              }
              
              return isValid;
            } catch (error) {
              return false;
            }
          });

          if (validData.length === 0) {
            alert('No valid data in CSV file');
            return;
          }

          const formattedData = validData.map((row) => ({
            id: row[Object.keys(row)[0]],
            url: row[Object.keys(row)[1]], // Update to match backend structure
          }));

          setCsvData(formattedData);
          setShowProgress(false);
          setShowCharts(false);
          setValidationMessage(
            `CSV file '${fileName}' uploaded and validated successfully. Removed ${nullRecordsCount} null records, ${duplicateRecordsCount} duplicate records. Total valid records: ${formattedData.length}`
          );
        },
      });
    };
    reader.readAsText(file);
  };

  const handleProcess = () => {
    if (!csvData) {
      alert('Please upload a CSV file before processing.');
      return;
    }
    setProcessing(true);
    // Send data to API
    fetch('http://localhost:8080/process', { // Update endpoint URL
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(csvData),
    })
      .then(response => {
        if (!response.ok) {
          throw new Error('Network response was not ok');
        }
  
        // Process the streaming data
        const reader = response.body.getReader();
        const decoder = new TextDecoder('utf-8');
  
        let totalRecordsProcessedLocal = 0;
        let successCountLocal = 0;
        let unsuccessCountLocal = 0;
        const resultsLocal = [];
  
        const readStream = async () => {
          try {
            while (true) {
              const { done, value } = await reader.read();
              if (done) {
                console.log('Stream complete');
                break;
              }
  
              const chunk = decoder.decode(value, { stream: true });
  
              // Split the chunk by newline to handle multiple JSON objects
              const updates = chunk.split('\n').filter(line => line.trim() !== '');
  
              updates.forEach(update => {
                try {
                  // If the update contains multiple JSON objects, split them
                  const jsonObjects = update.split('}{').map((jsonStr, index, array) => {
                    if (array.length > 1) {
                      if (index === 0) {
                        return jsonStr + '}';
                      } else if (index === array.length - 1) {
                        return '{' + jsonStr;
                      } else {
                        return '{' + jsonStr + '}';
                      }
                    }
                    return jsonStr;
                  });
  
                  jsonObjects.forEach(jsonStr => {
                    const parsedUpdate = JSON.parse(jsonStr);
                    console.log('Received update:', parsedUpdate);
  
                    totalRecordsProcessedLocal++;
                    if (parsedUpdate.result.response_code === 200) {
                      successCountLocal++;
                    } else {
                      unsuccessCountLocal++;
                    }
  
                    resultsLocal.push(parsedUpdate);
  
                    setProgress((totalRecordsProcessedLocal / parsedUpdate.total_records) * 100);
                    setTotalRecords(parsedUpdate.total_records);
                    setSuccessCount(successCountLocal);
                    setUnsuccessCount(unsuccessCountLocal);
                    setResults(resultsLocal);
                    setTotalRecordsProcessed(totalRecordsProcessedLocal);
                  });
                } catch (error) {
                  console.error('Error parsing update:', error);
                  console.log('Update content:', update); // Log the problematic update
                }
              });
            }
          } catch (error) {
            console.error('Stream read error:', error);
          }
          finally {
            setProcessing(false); // Show the process button again when processing completes
          }
        };
  
        readStream().catch(error => {
          console.error('Stream error:', error);
        });
  
        setShowCharts(true);
        setShowProgress(true);
      })
      .catch(error => {
        console.error('Error:', error);
        alert('Error processing CSV data, make sure API is running..');
        setProcessing(false);
      });
  };
  

  const responseCodeCounts = results.reduce((acc, result) => {
    acc[result.result.response_code] = (acc[result.result.response_code] || 0) + 1; // Adjust for the backend response structure
    return acc;
  }, {});

  const chartData = {
    labels: Object.keys(responseCodeCounts),
    datasets: [
      {
        label: '# of Responses',
        data: Object.values(responseCodeCounts),
        backgroundColor: [
          'rgba(75, 192, 192, 0.2)',
          'rgba(255, 99, 132, 0.2)',
          'rgba(54, 162, 235, 0.2)',
          'rgba(255, 206, 86, 0.2)',
          'rgba(75, 192, 192, 0.2)',
        ],
        borderColor: [
          'rgba(75, 192, 192, 1)',
          'rgba(255, 99, 132, 1)',
          'rgba(54, 162, 235, 1)',
          'rgba(255, 206, 86, 1)',
          'rgba(75, 192, 192, 1)',
        ],
        borderWidth: 1,
      },
    ],
  };

  const csvDownloadData = results.map(result => result.result); // Extract only the key-value pairs from parsedUpdate.result
// Inside your component
let buttonClass = 'button-default';

  return (
    <div className="App">
      <header className="App-header">
        <h1 className="project-name">AsyncCSVXpert</h1>
        <div
          className="file-drop"
          onDrop={handleFileDrop}
          onDragOver={(e) => e.preventDefault()}
          onClick={handleFileClick}
        >
          <p>Drag and drop a CSV file here or click to upload</p>
          <input
            type="file"
            accept=".csv"
            onChange={handleFileChange}
            ref={fileInputRef}
            style={{ display: 'none' }}
          />
        </div>
        {validationMessage && <p className="validation-message">{validationMessage}</p>}
        <button className="process-button" onClick={handleProcess} disabled={processing}>
            {processing ? 'Processing...' : 'Process CSV'}
        </button>
        {showProgress && (
          <div className="progress-bar">
            <div className="progress" style={{ width: `${progress}%` }}></div>
            <p>
              <span className="progress-text">{progress.toFixed(2)}% Processed: {totalRecordsProcessed} / {totalRecords}</span>
              {' '}
              <span className="success-text">Success: {successCount}</span>
              {' '}
              <span className="fail-text">Unsuccessful: {unsuccessCount}</span>
            </p>

          </div>
        )}
                 
        {showCharts && (
          <div className="charts">
            <Pie
              data={chartData}
              options={{ responsive: true, maintainAspectRatio: false }}
              width={400}
              height={400}
            />
          </div>
        )}
       <div>
    {results.length > 0 && (
      <CSVLink data={csvDownloadData} filename={fileName} className="download-link">
        <button className={`download-button ${buttonClass}`}>Download Results</button>
      </CSVLink>
    )}
  </div>
      </header>
    </div>
  );
}

export default App;
