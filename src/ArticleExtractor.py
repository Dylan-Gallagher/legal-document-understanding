from bs4 import BeautifulSoup
import re
import networkx as nx
from pyvis.network import Network

class ArticleExtractor:

    def __init__(self, config={}):
        if "gdpr_path" in config:
            self.gdpr_path = config["gdpr_path"]
        else:
            self.gdpr_path = "../../datasets/gdpr/gdpr.html"
        
        if "dga_path" in config:
            self.dga_path = config["dga_path"]
        else:
            self.dga_path = "../../datasets/dga/dga.html" 

        if "extraction_method" in config:
            if config["extraction_method"] not in ["internal", "external"]:
                raise ValueError("Invalid config.extraction_method. Must be 'internal' or 'external'.")
            self.extraction_method = config["extraction_method"]
        else:
            self.extraction_method = "internal" # Default to internal extraction
            
        self.legal_texts = None

      
    def parse_legal_texts(self):
        '''
        Parses the GDPR and DGA into a dictionary

        Returns a dictionary of legal texts.

        Usage:
            legal_texts = parse_legal_texts()

            # This is where the sentence corresponding to that point is
            legal_texts["GDPR"]["Article 2"]["Point 3"] 
        '''

        with open(self.gdpr_path, "r", encoding="utf-8") as f:
            gdpr_soup = BeautifulSoup(f, "html.parser")


        with open(self.dga_path, "r", encoding="utf-8") as f:
            dga_soup = BeautifulSoup(f, "html.parser")   


        legal_texts = {}
        
        pattern = re.compile(r'art_\d+(?!\.tit_1)$')
        gdpr_articles = gdpr_soup.find_all("div", id=pattern)
            
        # GDPR
        gdpr_dict = {}
        for article in gdpr_articles:
            article_dict = {}
            points = article.find_all("div", class_="norm")
            article_num = article['id'].split('_')[-1]

            for point in points:    # "norm" in the document
                has_sub_points = point.find("div") and point.find("div").find("p")
                if point.find("div"):
                    point_num = point.find('span').text.split('.')[0]

                if has_sub_points:
                    sub_point_dict = {}
                    for sub_point in point.find("div").find_all("div", class_=['grid-container', 'grid-list']):
                        suffix = None
                        if sub_point.find("div", class_="grid-list-column-2").find("p"):
                            suffix = sub_point.find("div", class_="grid-list-column-2").find("p").text
                        elif sub_point.find("div", class_="grid-list-column-2").find("div"):
                            suffix = sub_point.find("div", class_="grid-list-column-2").find("div").text
                        sentence = (point.find("div").find("p").text + " " +
                                    suffix)

                        sub_point_num = sub_point.find('div').find('span').text.strip()
                        if sentence:
                            sub_point_dict[f"Subpoint {sub_point_num}"] = sentence
                    article_dict[f"Point {point_num}"] = sub_point_dict

                elif point.find("div"):
                    sentence = point.find("div").text

                    if sentence:
                        article_dict[f"Point {point_num}"] = sentence
            gdpr_dict[f"Article {article_num}"] = article_dict

        legal_texts["GDPR"] = gdpr_dict

        # Data Governance Act (DGA)
        dga_articles = dga_soup.find_all("div", id=pattern)

        dga_dict = {}
        
        dga_articles = dga_soup.find_all("div", id=pattern)

        for article in dga_articles:
            article_dict = {}
            points = article.find_all("div", id=re.compile(r'\d+\.\d+'))
            article_num = article['id'].split('_')[-1]

            for point in points:
                point_num = int(point['id'].split('.')[-1])
                point_content = point.find("p", class_="oj-normal")
                if point_content:
                    point_text = re.sub(r'^\d+\.\s*', '', point_content.text).strip()
                    article_dict[f"Point {point_num}"] = point_text

                sub_points = point.find_all("table")
                sub_point_dict = {}
                for sub_point in sub_points:
                    sub_point_num = sub_point.find("p", class_="oj-normal").text.strip('() ')
                    sub_point_content = sub_point.find("td", valign="top").find_next_sibling("td").find("p", class_="oj-normal")
                    if sub_point_content:
                        sub_point_text = f"{point_text} {sub_point_content.text.strip()}"
                        sub_point_dict[f"Subpoint ({sub_point_num})"] = sub_point_text
                if sub_point_dict:
                    article_dict[f"Point {point_num}"] = sub_point_dict

            dga_dict[f"Article {article_num}"] = article_dict

        legal_texts["DGA"] = dga_dict

        self.legal_texts = legal_texts

        return legal_texts

    def find_internal_links_in_legal_text(self):
        """
        Look for all mentions of "Article X" within the GDPR.
        """
        references = []
        
        pattern = re.compile(r"Article \d+(\(\d+\))?([a-z])?")
        # pattern = re.compile(r"Article \d+")

        for article_name, article_dict in self.legal_texts["GDPR"].items():
            for point_name, point_val in article_dict.items():
                if type(point_val) is dict:
                    # It has subpoints
                    for subpoint_name, sentence in point_val.items():
                        match = re.search(pattern, sentence)
                        if match:  # This only matches the first one, look into matching multiple Articles
                            references.append({"head": article_name, "tail": match.group()})
                else:
                    # It doesn't have subpoints
                    match = re.search(pattern, point_val)
                    if match:  # This only matches the first one, look into matching multiple Articles
                        references.append({"head": article_name, "tail": match.group()})
        
        self.references = references
        return references

    def find_external_links_in_legal_text(self):
        """
        Look for all mentions of GDPR (Regulation (EU) 2016/679) in the DGA
        """
        references = []
        for article_name, article_dict in self.legal_texts["DGA"].items():
            for point_name, point_val in article_dict.items():
                if type(point_val) is dict:
                    # It has subpoints
                    for subpoint_name, sentence in point_val.items():
                        if "Regulation (EU) 2016/679" in sentence:
                            references.append({"head": "DGA: " + article_name + ", " + point_name, "tail": "GDPR"})
                else:
                    # It doesn't have subpoints
                    if "Regulation (EU) 2016/679" in point_val:
                        references.append({"head": "DGA: " + article_name + ", " + point_name, "tail": "GDPR"})

        self.references = references
        return references

    def run(self, article_linking="internal", draw_graph=True):
        """
        Run article extraction

        params:
            article_linking: ["internal", "external"]
                "internal": Article mentions within a document 
                e.g. Article 23 mentions Article 42 (within the same document)

                "external": Article mentions to another document
                e.g Article 25 mentions GDPR
        
        returns:
            references: A dict of relations/references representing the article mentions
        """

        self.parse_legal_texts()

        if self.extraction_method == "internal":
            self.find_internal_links_in_legal_text()
        elif self.extraction_method == "external":
            self.find_external_links_in_legal_text()
        
        if draw_graph:
            G = nx.DiGraph()

            # Iterate through data to add edges to the graph
            for item in self.references:
                G.add_edge(item["head"], item["tail"])

            # Initialize PyVis network
            net = Network(notebook=False, height="750px", width="100%")
            net.from_nx(G)

            for edge in net.edges:
                edge["arrows"] = "to"

            # Customize the visualization
            net.show_buttons(filter_=['physics'])
            net.toggle_physics(True)

            # Save the visualization to a html file
            net.show("legal_document_relations.html")

        
        

        

        






