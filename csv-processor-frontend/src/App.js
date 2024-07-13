import React, { useState, useRef, useEffect } from 'react';
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
  const [showProgress, setShowProgress] = useState(false);
  const [showCharts, setShowCharts] = useState(false);
  const [validationMessage, setValidationMessage] = useState('');
  const fileInputRef = useRef(null);

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
            URL: row[Object.keys(row)[1]],
          }));

          setCsvData(formattedData);
          setShowProgress(false);
          setShowCharts(false);
          setValidationMessage(
            `CSV file uploaded and validated successfully. Removed ${nullRecordsCount} null records, ${duplicateRecordsCount} duplicate records. Total valid records: ${formattedData.length}`
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
    
    // Send data to API
    fetch('http://localhost:8080/process-csv', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(csvData),
    })
    .then(response => response.json())
    .then(data => {
      if (data.status === 'ok') {
        setShowCharts(true);
        setShowProgress(true);
        getRealTimeUpdates();
      } else {
        alert('Error processing CSV data');
      }
    })
    .catch(error => {
      console.error('Error:', error);
      alert('Error processing CSV data');
    });
  };

  const getRealTimeUpdates = () => {
    const interval = setInterval(() => {
      fetch('http://localhost:8080/progress')
        .then(response => response.json())
        .then(data => {
          setProgress(data.progress);
          setResults(data.results);
          if (data.progress >= 100) {
            clearInterval(interval);
            setShowProgress(false);
          }
        })
        .catch(error => {
          console.error('Error:', error);
          clearInterval(interval);
        });
    }, 1000);
  };

  const responseCodeCounts = results.reduce((acc, result) => {
    acc[result.responseCode] = (acc[result.responseCode] || 0) + 1;
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
        {showProgress && (
          <div className="progress-bar">
            <div className="progress" style={{ width: `${progress}%` }}></div>
          </div>
        )}
        <button onClick={handleProcess} className="process-button">
          Process
        </button>
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
        {results.length > 0 && showCharts && (
          <CSVLink data={results} filename="results.csv" className="App-link">
            Download Results
          </CSVLink>
        )}
      </header>
    </div>
  );
}

export default App;
