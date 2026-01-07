<?php declare(strict_types=1);
# Borrowed from https://github.com/DaveRandom/Resume/

final class DefaultOutputWriter implements OutputWriter
{
    /**
     * {@inheritdoc}
     * @codeCoverageIgnore
     */
    public function setResponseCode(int $code)
    {
        \header("HTTP/1.1 {$code} " . self::RESPONSE_MESSAGES[$code]);
    }

    /**
     * {@inheritdoc}
     * @codeCoverageIgnore
     */
    public function sendHeader(string $name, string $value)
    {
        \header("{$name}: {$value}");
    }

    /**
     * {@inheritdoc}
     */
    public function sendData(string $data)
    {
        echo $data;
    }
}




final class FileResource implements Resource
{
    /** @internal */
    const DEFAULT_CHUNK_SIZE = 8192;

    /**
     * Full canonical path of file on local file system
     *
     * @var string
     */
    private $localPath;

    /**
     * MIME type of file contents
     *
     * @var string
     */
    private $mimeType;

    /**
     * Size of local file, in bytes
     *
     * @var int
     */
    private $fileSize;

    /**
     * Stream handle for reading the file
     *
     * @var resource|null
     */
    private $handle = null;

    /**
     * Chunk size for local file system reads when sending a partial file
     *
     * @var int
     */
    private $chunkSize = self::DEFAULT_CHUNK_SIZE;

    /**
     * Open the local file handle if it's not open yet, and set the pointer to the supplied position
     *
     * @param int $position
     */
    private function openFile(int $position)
    {
        if ($this->handle === null && !$this->handle = \fopen($this->localPath, 'r')) {
            throw new SendFileFailureException("Failed to open '{$this->localPath}' for reading");
        }

        if (\fseek($this->handle, $position, \SEEK_SET) !== 0) {
            throw new SendFileFailureException('fseek() operation failed');
        }
    }

    /**
     * Send a chunk of data to the client
     *
     * @param OutputWriter $outputWriter
     * @param int $length
     * @return int
     */
    private function sendDataChunk(OutputWriter $outputWriter, int $length): int
    {
        $read = $length > $this->chunkSize
            ? $this->chunkSize
            : $length;

        $data = \fread($this->handle, $read);

        if ($data === false) {
            throw new SendFileFailureException('fread() operation failed');
        }

        $outputWriter->sendData($data);

        return \strlen($data);
    }

    /**
     * @param string $path Path of file on local file system
     * @param string $mimeType MIME type of file contents
     * @param int $chunkSize Chunk size for local file system reads when sending a partial file
     */
    public function __construct(string $path, string $mimeType = null, int $chunkSize = self::DEFAULT_CHUNK_SIZE)
    {
        $this->chunkSize = $chunkSize;
        $this->mimeType = $mimeType ?? 'application/octet-stream';

        // Make sure the file exists and is a file, otherwise we are wasting our time
        $this->localPath = \realpath($path);

        if ($this->localPath === false || !\is_file($this->localPath)) {
            throw new NonExistentFileException("Local path '{$path}' does not exist or is not a file");
        }

        // This shouldn't ever fail but just in case
        if (false === $this->fileSize = \filesize($this->localPath)) {
            throw new UnreadableFileException("Failed to retrieve size of file '{$this->localPath}'");
        }
    }

    /**
     * Explicitly close the file handle if it's open
     */
    public function __destruct()
    {
        if ($this->handle !== null) {
            \fclose($this->handle);
        }
    }

    /**
     * {@inheritdoc}
     */
    public function sendData(OutputWriter $outputWriter, Range $range = null, string $unit = null)
    {
        if (\strtolower($unit ?? 'bytes') !== 'bytes') {
            throw new UnsatisfiableRangeException('Unit not handled by this resource: ' . $unit);
        }

        $start = 0;
        $length = $this->fileSize;

        if ($range !== null) {
            $start = $range->getStart();
            $length = $range->getLength();
        }

        $this->openFile($start);

        while ($length > 0) {
            $length -= $this->sendDataChunk($outputWriter, $length);
        }
    }

    /**
     * {@inheritdoc}
     */
    public function getLength(): int
    {
        return $this->fileSize;
    }

    /**
     * {@inheritdoc}
     */
    public function getMimeType(): string
    {
        return $this->mimeType;
    }

    /**
     * {@inheritdoc}
     */
    public function getAdditionalHeaders(): array
    {
        return [
            'Content-Disposition' => 'attachment; filename="' . \basename($this->localPath) . '"'
        ];
    }

    /**
     * Get the chunk size for local file system reads when sending a partial file
     *
     * @return int
     */
    public function getChunkSize(): int
    {
        return $this->chunkSize;
    }

    /**
     * Set the chunk size for local file system reads when sending a partial file
     *
     * @param int $chunkSize
     */
    public function setChunkSize(int $chunkSize)
    {
        $this->chunkSize = $chunkSize;
    }
}




final class HeaderSet implements \IteratorAggregate
{
    private $keyMap;
    private $values;

    public function __construct(array $headers = [])
    {
        foreach ($headers as $name => $value) {
            $this->setHeader($name, $value);
        }
    }

    public function getIterator(): \Iterator
    {
        return new \ArrayIterator($this->values);
    }

    public function setHeader(string $name, string $value)
    {
        $key = \strtolower($name);
        $name = $this->keyMap[$key] ?? $name;

        $this->values[$name] = $value;
        $this->keyMap[$key] = $name;
    }

    public function getHeader(string $name)
    {
        return $this->values[$this->keyMap[\strtolower($name)]] ?? null;
    }

    public function containsHeader(string $name): bool
    {
        return isset($this->values[$this->keyMap[\strtolower($name)]]);
    }

    public function removeHeader(string $name)
    {
        $key = \strtolower($name);
        $name = $this->keyMap[$key] ?? $name;

        unset($this->values[$name], $this->keyMap[$key]);
    }
}




final class IncompatibleRangesException extends LogicException { }




final class InvalidRangeException extends \LogicException { }




final class InvalidRangeHeaderException extends RuntimeException { }




final class LengthNotAvailableException extends LogicException { }




// abstract class LogicException extends \LogicException { }




final class NonExistentFileException extends RuntimeException { }




interface OutputWriter
{
    const RESPONSE_MESSAGES = [
        200 => 'OK',
        206 => 'Partial Content',
    ];

    /**
     * Set the HTTP response code to send to the client
     *
     * @param int $code
     */
    function setResponseCode(int $code);

    /**
     * Send a response header to the client
     *
     * @param string $name
     * @param string $value
     */
    function sendHeader(string $name, string $value);

    /**
     * Send a data block to the client
     *
     * @param string $data
     */
    function sendData(string $data);
}




final class Range
{
    private $start;
    private $end;
    private $normal;

    public function __construct(int $start, int $end = null)
    {
        $this->start = $start;
        $this->end = $end;

        if ($end < 0) {
            throw new InvalidRangeException('End cannot be negative');
        }

        $haveEnd = $end !== null;

        if ($haveEnd && $start > $end) {
            throw new InvalidRangeException('Start cannot be larger than end');
        }

        if ($haveEnd && $start < 0) {
            throw new InvalidRangeException('A range with a negative start cannot specify an end');
        }

        $this->normal = $start >= 0 && $haveEnd;
    }

    public function getStart(): int
    {
        return $this->start;
    }

    /**
     * @return int|null
     */
    public function getEnd()
    {
        return $this->end;
    }

    public function getLength(): int
    {
        if (!$this->normal) {
            throw new LengthNotAvailableException('Cannot retrieve length of a range that is not normalized');
        }

        return ($this->end - $this->start) + 1;
    }

    public function normalize(int $size): Range
    {
        if ($this->normal) {
            if ($this->start > $size) {
                throw new UnsatisfiableRangeException('Not satisfiable by a resource of the specified size');
            }

            return $this;
        }

        $end = $this->end ?? $size - 1;
        $start = $this->start < 0
            ? $end + $this->start + 1
            : $this->start;

        if ($start > $size) {
            throw new UnsatisfiableRangeException('Not satisfiable by a resource of the specified size');
        }

        return new self(\max($start, 0), \min($end, $size - 1));
    }

    public function overlaps(Range $other): bool
    {
        if (!$this->normal || !$other->normal) {
            throw new IncompatibleRangesException('Cannot test for overlap of ranges that have not been normalized');
        }

        // https://stackoverflow.com/a/3269471/889949
        return $this->start <= $other->end && $other->start <= $this->end;
    }

    public function combine(Range $other): self
    {
        if (!$this->normal || !$other->normal) {
            throw new IncompatibleRangesException('Cannot combine ranges that have not been normalized');
        }

        if (!($this->start <= $other->end && $other->start <= $this->end)) {
            throw new IncompatibleRangesException('Cannot combine non-overlapping ranges');
        }

        return new self(\min($this->start, $other->start), \max($this->end, $other->end));
    }

    public function __toString(): string
    {
        $suffix = $this->end !== null || $this->start >= 0
            ? '-' . $this->end
            : '';

        return $this->start . $suffix;
    }
}




final class RangeNotApplicableException extends LogicException { }




final class RangeSet
{
    const DEFAULT_MAX_RANGES = 10;

    /** @internal */
    const HEADER_PARSE_EXPR = /** @lang regex */ '/
      ^
      \s*                 # tolerate lead white-space
      (?<unit> [^\s=]+ )  # unit is everything up to first = or white-space
      (?: \s*=\s* | \s+ ) # separator is = or white-space
      (?<ranges> .+ )     # remainder is range spec
    /x';

    /** @internal */
    const RANGE_PARSE_EXPR = /** @lang regex */ '/
      ^
      (?<start> [0-9]* ) # start is a decimal number
      \s*-\s*            # separator is a dash
      (?<end> [0-9]* )   # end is a decimal number
      $
    /x';

    /**
     * The unit for ranges in the set
     *
     * @var string
     */
    private $unit;

    /**
     * The ranges in the set
     *
     * @var Range[]
     */
    private $ranges = [];

    /**
     * Parse an array of range specifiers into an array of Range objects
     *
     * @param string[] $ranges
     * @return Range[]
     */
    private static function parseRanges(array $ranges): array
    {
        $result = [];

        foreach ($ranges as $i => $range) {
            if (!\preg_match(self::RANGE_PARSE_EXPR, \trim($range), $match)) {
                throw new InvalidRangeHeaderException("Invalid range format at position {$i}: Parse failure");
            }

            if ($match['start'] === '' && $match['end'] === '') {
                throw new InvalidRangeHeaderException("Invalid range format at position {$i}: Start and end empty");
            }

            $result[] = $match['start'] === ''
                ? new Range(((int)$match['end']) * -1)
                : new Range((int)$match['start'], $match['end'] !== '' ? (int)$match['end'] : null);
        }

        return $result;
    }

    /**
     * Get a set of normalized ranges applied to a resource size
     *
     * @param int $size
     * @return Range[]
     */
    private function normalizeRangesForSize(int $size): array
    {
        $result = [];

        foreach ($this->ranges as $range) {
            try {
                $range = $range->normalize($size);

                if ($range->getStart() < $size) {
                    $result[] = $range;
                }
            } catch (UnsatisfiableRangeException $e) {
                // ignore, other ranges in the set may be satisfiable
            }
        }

        if (empty($result)) {
            throw new UnsatisfiableRangeException('No specified ranges are satisfiable by a resource of the specified size');
        }

        return $result;
    }

    /**
     * Combine overlapping ranges in the supplied array and return the result
     *
     * @param Range[] $ranges
     * @return Range[]
     */
    private function combineOverlappingRanges(array $ranges)
    {
        \usort($ranges, static function(Range $a, Range $b) {
            return $a->getStart() <=> $b->getStart();
        });

        for ($i = 0, $l = \count($ranges) - 1; $i < $l; $i++) {
            if (!$ranges[$i]->overlaps($ranges[$i + 1])) {
                continue;
            }

            $ranges[$i] = $ranges[$i]->combine($ranges[$i + 1]);
            unset($ranges[$i + 1]);

            $i++;
        }

        return $ranges;
    }

    /**
     * Create a new instance from a Range header string
     *
     * @param string|null $header
     * @param int $maxRanges
     * @return self|null
     */
    public static function createFromHeader(string $header = null, int $maxRanges = self::DEFAULT_MAX_RANGES)
    {
        if ($header === null) {
            return null;
        }

        if (!\preg_match(self::HEADER_PARSE_EXPR, $header, $match)) {
            throw new InvalidRangeHeaderException('Invalid header: Parse failure');
        }

        $unit = $match['unit'];
        $ranges = \explode(',', $match['ranges']);

        if (\count($ranges) > $maxRanges) {
            throw new InvalidRangeHeaderException("Invalid header: Too many ranges");
        }

        return new self($unit, self::parseRanges($ranges));
    }

    /**
     * @param string $unit
     * @param Range[] $ranges
     */
    public function __construct(string $unit, array $ranges)
    {
        $this->unit = $unit;
        $this->ranges = $ranges;
    }

    /**
     * Get the unit for ranges in the set
     *
     * @return string
     */
    public function getUnit(): string
    {
        return $this->unit;
    }

    /**
     * Get a set of normalized ranges applied to a resource size, reduced to the minimum set of ranges
     *
     * @param int $size
     * @return Range[]
     */
    public function getRangesForSize(int $size): array
    {
        $ranges = $this->normalizeRangesForSize($size);

        $previousCount = null;
        $count = \count($ranges);

        while ($count > 1 && $count !== $previousCount) {
            $previousCount = $count;

            $ranges = $this->combineOverlappingRanges($ranges);

            $count = \count($ranges);
        }

        return $ranges;
    }
}




interface RangeUnitProvider extends Resource
{
    /**
     * Get a list of the range units supported by this resource
     *
     * @return string[]
     */
    function getRangeUnits(): array;
}




interface Resource
{
    /**
     * Write the specified data range to output
     *
     * @param OutputWriter $outputWriter
     * @param string|null $unit
     * @param Range|null $range
     */
    function sendData(OutputWriter $outputWriter, Range $range = null, string $unit = null);

    /**
     * Get the total length of the resource
     *
     * @return int
     */
    function getLength(): int;

    /**
     * Get the MIME type of the resource
     *
     * @return string
     */
    function getMimeType(): string;

    /**
     * Get additional headers to be send with the resource
     *
     * @return array
     */
    function getAdditionalHeaders(): array;
}




final class ResourceServlet
{
    /**
     * @var \DaveRandom\Resume\Resource
     */
    private $resource;

    /**
     * Generate the default response headers for this resource
     *
     * @return HeaderSet
     */
    private function generateDefaultHeaders(): HeaderSet
    {
        $ranges = $this->resource instanceof RangeUnitProvider
            ? \implode(',', $this->resource->getRangeUnits())
            : 'bytes';

        if ($ranges === '') {
            $ranges = 'none';
        }

        return new HeaderSet([
            'Content-Type' => $this->resource->getMimeType(),
            'Accept-Ranges' => $ranges,
        ]);
    }

    /**
     * Send the headers that are included regardless of whether a range was requested
     *
     * @param OutputWriter $outputWriter
     * @param HeaderSet $headers
     */
    private function sendHeaders(OutputWriter $outputWriter, HeaderSet $headers)
    {
        foreach ($this->resource->getAdditionalHeaders() as $name => $value) {
            $headers->setHeader($name, $value);
        }

        foreach ($headers as $name => $value) {
            $outputWriter->sendHeader(\trim($name), \trim($value));
        }
    }

    /**
     * Create a Content-Range header corresponding to the specified unit and ranges
     *
     * @param string $unit
     * @param Range[] $ranges
     * @param int $size
     * @return string
     */
    private function getContentRangeHeader(string $unit, array $ranges, int $size): string
    {
        return $unit . ' ' . \implode(',', $ranges) . '/' . $size;
    }

    /**
     * Send the complete resource to the client
     *
     * @param OutputWriter $outputWriter
     * @param HeaderSet $headers
     */
    private function sendCompleteResource(OutputWriter $outputWriter, HeaderSet $headers)
    {
        $outputWriter->setResponseCode(200);

        $this->sendHeaders($outputWriter, $headers);

        $headers->setHeader('Content-Length', (string)$this->resource->getLength());

        $this->resource->sendData($outputWriter);
    }

    /**
     * Send the requested ranges to the client
     *
     * @param OutputWriter $outputWriter
     * @param HeaderSet $headers
     * @param RangeSet $rangeSet
     */
    private function sendResourceRanges(OutputWriter $outputWriter, HeaderSet $headers, RangeSet $rangeSet)
    {
        $totalResourceSize = $this->resource->getLength();
        $ranges = $rangeSet->getRangesForSize($totalResourceSize);

        $responseBodySize = \array_reduce($ranges, function(int $size, Range $range) {
            return $size + $range->getLength();
        }, 0);

        $outputWriter->setResponseCode(206);
        $this->sendHeaders($outputWriter, $headers);

        $contentRangeHeader = $this->getContentRangeHeader($rangeSet->getUnit(), $ranges, $totalResourceSize);

        $outputWriter->sendHeader('Content-Range', $contentRangeHeader);
        $outputWriter->sendHeader('Content-Length', (string)$responseBodySize);

        foreach ($ranges as $range) {
            $this->resource->sendData($outputWriter, $range);
        }
    }

    public function __construct(Resource $resource)
    {
        $this->resource = $resource;
    }

    /**
     * Send data from a file based on the Range header described by the supplied RangeSet
     *
     * @param RangeSet|null $rangeSet Range header on which the transmission will be based
     * @param OutputWriter|null $outputWriter Output writer via which resource will be sent
     */
    public function sendResource(RangeSet $rangeSet = null, OutputWriter $outputWriter = null)
    {
        $outputWriter = $outputWriter ?? new DefaultOutputWriter();
        $headers = $this->generateDefaultHeaders();

        if ($rangeSet === null) {
            $this->sendCompleteResource($outputWriter, $headers);
        } else {
            $this->sendResourceRanges($outputWriter, $headers, $rangeSet);
        }
    }
}








final class SendFileFailureException extends RuntimeException { }




final class UnreadableFileException extends RuntimeException { }




final class UnsatisfiableRangeException extends RuntimeException { }




/**
 * Get the value of a header in the current request context
 *
 * @param string $name Name of the header
 * @return string|null Returns null when the header was not sent or cannot be retrieved
 * @codeCoverageIgnore
 */
function get_request_header(string $name)
{
    $name = \strtoupper($name);

    // IIS/Some Apache versions and configurations
    if (isset($_SERVER['HTTP_' . $name])) {
        return \trim($_SERVER['HTTP_' . $name]);
    }

    // Various other SAPIs
    if (\function_exists('apache_request_headers')) {
        foreach (\apache_request_headers() as $headerName => $value) {
            if (\strtoupper($headerName) === $name) {
                return \trim($value);
            }
        }
    }

    return null;
}

?>