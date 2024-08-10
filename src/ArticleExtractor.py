from bs4 import BeautifulSoup
import re
import networkx as nx
from pyvis.network import Network
import os

class ArticleExtractor:

    def __init__(self, config={}):
        if "ai_act" in config:
            self.ai_act_path = config["ai_act"]
        else:
            self.ai_act_path = "../datasets/ai_act/ai_act.html" 

        if "gdpr_path" in config:
            self.gdpr_path = config["gdpr_path"]
        else:
            self.gdpr_path = "../datasets/gdpr/gdpr.html"
        
        if "dga_path" in config:
            self.dga_path = config["dga_path"]
        else:
            self.dga_path = "../datasets/dga/dga.html" 

        if "extraction_method" in config:
            if config["extraction_method"] not in ["internal", "external"]:
                raise ValueError("Invalid config.extraction_method. Must be 'internal' or 'external'.")
            self.extraction_method = config["extraction_method"]
        else:
            self.extraction_method = "external" # Default to internal extraction
            
        self.legal_texts = None

      
    def parse_legal_texts(self):
        '''
        Parses the AI Act, GDPR, and DGA into a dictionary

        Returns a dictionary of legal texts.

        Usage:
            legal_texts = parse_legal_texts()

            # This is where the sentence corresponding to that point is
            legal_texts["GDPR"]["Article 2"]["Point 3"] 
        '''

        with open(self.ai_act_path, "r", encoding="utf-8") as f:
            ai_act_soup = BeautifulSoup(f, "html.parser")

        with open(self.gdpr_path, "r", encoding="utf-8") as f:
            gdpr_soup = BeautifulSoup(f, "html.parser")

        with open(self.dga_path, "r", encoding="utf-8") as f:
            dga_soup = BeautifulSoup(f, "html.parser")

        legal_texts = {}
        
        pattern = re.compile(r'art_\d+(?!\.tit_1)$')
    

        # AI Act
        # Find all div elements with id matching the pattern xxx.xxx
        ai_act_divs = ai_act_soup.find_all('div', id=re.compile(r'^\d{3}\.\d{3}$'))
        ai_act_dict = {}
        for div in ai_act_divs:
            article_num, point_num = div['id'].split('.')
            point_num = int(point_num)
            article_num = int(article_num)
            
            if f"Article {article_num}" not in ai_act_dict.keys():
                ai_act_dict[f"Article {article_num}"] = {}

            
            # Check if the point has subpoints
            subpoints = div.find_all('table')
            
            if subpoints:
                main_text = div.find('p', class_='oj-normal').text.strip().split()[1:]
                main_text = " ".join(main_text)
                point_content = {
                    "subpoints": {}
                }
                
                for table in subpoints:
                    subpoint_letter = table.find('p', class_='oj-normal').text.strip()
                    subpoint_text = table.find_all('p', class_='oj-normal')[1].text.strip().split()[1:]
                    subpoint_text = " ".join(subpoint_text)
                    point_content["subpoints"][f"Subpoint {subpoint_letter}"] = main_text + subpoint_text
                
                ai_act_dict[f"Article {article_num}"][f"Point {point_num}"] = point_content
            else:
                point_text = div.find('p', class_='oj-normal').text.strip().split()[1:]
                point_text = " ".join(point_text)
                ai_act_dict[f"Article {article_num}"][f"Point {point_num}"] = point_text

        legal_texts["AI Act"] = ai_act_dict
                
        # GDPR
        gdpr_articles = gdpr_soup.find_all("div", id=pattern)
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

    def find_external_links_in_legal_text(self, source_doc, target_doc):
        """
        Look for all mentions of GDPR (Regulation (EU) 2016/679) in the DGA
        Look for all mentions of `target_doc` in the `source_doc`
        """

        official_name = {
            "GDPR": "Regulation (EU) 2016/679",
            "DGA": "Regulation (EU) 2018/1724",
            "AI Act": "Regulation(EU) 2024/1689",
        }

        references = []
        for article_name, article_dict in self.legal_texts[source_doc].items():
            for point_name, point_val in article_dict.items():
                if type(point_val) is dict:
                    # It has subpoints
                    for subpoint_name, sentence in point_val.items():
                        if official_name[target_doc] in sentence:
                            references.append({"head": f"{source_doc}: " + article_name + ", " + point_name, "tail": target_doc})
                else:
                    # It doesn't have subpoints
                    if official_name[target_doc] in point_val:
                        references.append({"head": f"{source_doc}: " + article_name + ", " + point_name, "tail": target_doc})

        self.references = references
        return references

    def run(self, source_doc="DGA", target_doc="GDPR", draw_graph=True):
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
            self.find_external_links_in_legal_text(source_doc, target_doc)
        
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
            if not os.path.exists("templates"):
                os.makedirs("templates")

            net.save_graph("templates/legal_document_relations.html")

if __name__ == "__main__":
    article_extractor = ArticleExtractor({"extraction_method": "external"})
    article_extractor.run()
